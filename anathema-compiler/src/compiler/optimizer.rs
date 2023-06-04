use crate::parsing::parser::Expression as ParseExpr;

enum ControlFlow {
    If(usize),
    Else(Option<usize>),
}

#[derive(Debug, PartialEq, Clone, Eq)]
pub(crate) enum Expression {
    If { cond: usize, size: usize },
    Else { cond: Option<usize>, size: usize },
    For { data: usize, binding: usize, size: usize },
    LoadText(usize),
    LoadAttribute { key: usize, value: usize },
    Node { ident: usize, scope_size: usize },
}

pub(crate) struct Optimizer {
    output: Vec<Expression>,
    input: Vec<ParseExpr>,
    ep: usize,
}

impl Optimizer {
    pub(crate) fn new(input: Vec<ParseExpr>) -> Self {
        Self { output: vec![], input, ep: 0 }
    }

    // -----------------------------------------------------------------------------
    //     - Optimize -
    //
    //     * Collapse empty if to just the body of the else if it exists
    //     * Remove empty else
    //     * Remove empty if
    //     * Remove empty for-loops
    //
    //     Possible future optimizations
    //     * Attribute keys could be string slices
    //     * Node idents could also be looked up beforehand
    // -----------------------------------------------------------------------------

    pub(crate) fn optimize(mut self) -> Vec<Expression> {
        self.remove_empty_if_else_for();

        while let Some(in_expr) = self.input.get(self.ep) {
            self.ep += 1;
            let out_expr = match in_expr {
                &ParseExpr::If(cond) => {
                    self.opt_control_flow(ControlFlow::If(cond));
                    continue;
                }
                &ParseExpr::Else(cond) => {
                    self.opt_control_flow(ControlFlow::Else(cond));
                    continue;
                }
                ParseExpr::ScopeStart => unreachable!("this should not happen as scopes are consumed by other expressions"),
                &ParseExpr::For { data, binding } => {
                    self.opt_for(data, binding);
                    continue
                }
                &ParseExpr::Node(ident_index) => {
                    let start = self.output.len();

                    // Get attributes and text
                    let mut text_and_attributes = 0;
                    loop {
                        match self.input.get(self.ep) {
                            Some(&ParseExpr::LoadText(index)) => {
                                self.output.push(Expression::LoadText(index));
                                text_and_attributes += 1;
                                self.ep += 1;
                            }
                            Some(&ParseExpr::LoadAttribute { key, value }) => {
                                self.output.push(Expression::LoadAttribute { key, value });
                                text_and_attributes += 1;
                                self.ep += 1;
                            }
                            _ => break
                        }
                    }

                    let child_scope_size = match self.input.get(self.ep) {
                        Some(ParseExpr::ScopeStart) => {
                            self.opt_scope();
                            self.output.len() - start - text_and_attributes
                        }
                        _ => 0

                    };
                    self.output.insert(start, Expression::Node { ident: ident_index, scope_size: child_scope_size });
                    continue
                }
                &ParseExpr::LoadText(index) => Expression::LoadText(index),
                &ParseExpr::LoadAttribute { key, value } => Expression::LoadAttribute { key, value },
                ParseExpr::EOF => continue, // noop, we don't care about EOF
                ParseExpr::ScopeEnd => unreachable!("scopes are consumed by `opt_scope`"),
            };

            self.output.push(out_expr);
        }

        self.output
    }

    fn opt_control_flow(&mut self, control_flow: ControlFlow) {
        let start = self.output.len();
        self.opt_scope();
        let size = self.output.len() - start;
        let expr = match control_flow {
            ControlFlow::If(cond) => Expression::If { cond, size },
            ControlFlow::Else(cond) => Expression::Else { cond, size },
        };
        self.output.insert(start, expr);
    }

    fn opt_scope(&mut self) {
        if let Some(ParseExpr::ScopeStart) = self.input.get(self.ep) {
            self.ep += 1; // consume ScopeStart
        } else {
            todo!("invalid expression: {:?}", self.input.get(self.ep)); // TODO: should all this return results?
        };

        let start = self.ep;
        let mut end = self.ep;
        let mut level = 1;

        while let Some(expr) = self.input.get(end) {
            match expr {
                ParseExpr::ScopeStart => level += 1,
                ParseExpr::ScopeEnd => {
                    level -= 1;
                    if level == 0 {
                        let input = self.input.drain(start..end).collect::<Vec<_>>();
                        self.ep += 1; // consume the ScopeEnd
                        let mut output = Optimizer::new(input).optimize();
                        self.output.append(&mut output);
                        break;
                    }
                }
                _ => {}
            }
            end += 1;
        }
    }

    fn opt_for(&mut self, data: usize, binding: usize) {
        let start = self.output.len();
        self.opt_scope();
        let end = self.output.len();
        self.output.insert(start, Expression::For { data, binding, size: end - start });
    }

    fn remove_empty_if_else_for(&mut self) {
        let mut p = 0;
        while let Some(expr) = self.input.get(p) {
            if let ParseExpr::If(_) | ParseExpr::Else(_) | ParseExpr::For { .. } = expr {
                match self.input.get(p + 1) {
                    Some(ParseExpr::ScopeStart) => p += 1,
                    _ => drop(self.input.remove(p)),
                }
            } else {
                p += 1;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::lexer::Lexer;
    use crate::parsing::parser::Parser;
    use crate::parsing::Constants;

    fn parse(src: &str) -> Vec<Expression> {
        let mut ctx = Constants::default();
        let lexer = Lexer::new(src);
        let parser = Parser::new(lexer, &mut ctx).unwrap();
        let expr = parser.map(|e| e.unwrap()).collect();
        let opt = Optimizer::new(expr);
        opt.optimize()
    }

    #[test]
    fn optimize_nested_scopes() {
        let src = "
        a
            a
            ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 1 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
    }

    #[test]
    fn optimize_if() {
        let src = "
        if {{ a }}
            a
            ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::If { cond: 0, size: 1 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
    }

    #[test]
    fn optimize_else() {
        let src = "
        if {{ a }}
            a
        else
            a
            ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::If { cond: 0, size: 1 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert_eq!(expressions.remove(0), Expression::Else { cond: None, size: 1 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
    }

    #[test]
    fn optimize_for() {
        let src = "
        a
        for b in {{ b }}
            a
            b
            ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::Node{ ident: 0, scope_size: 0 });
        assert_eq!(expressions.remove(0), Expression::For { data: 0, binding: 1, size: 2 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 1, scope_size: 0 });
    }

    #[test]
    fn nested_ifs() {
        let src = "
        if {{ a }}
            if {{ a }}
                a
            ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::If { cond: 0, size: 2 });
        assert_eq!(expressions.remove(0), Expression::If { cond: 0, size: 1 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
    }

    #[test]
    fn remove_empty_elses() {
        let src = "
        if {{ x }}
            a
            a
        else
        if {{ x }}
            a
        else
        b
        ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::If { cond: 0, size: 2 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert_eq!(expressions.remove(0), Expression::If { cond: 0, size: 1 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 1, scope_size: 0 });
    }

    #[test]
    fn remove_empty_if() {
        let src = "
        if {{ data }}
        x
        ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert!(expressions.is_empty());
    }

    #[test]
    fn remove_empty_else() {
        let src = "
            if {{ x }}
                x
            else
            x
        ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::If { cond: 0, size: 1 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert!(expressions.is_empty());
    }

    #[test]
    fn optimise_empty_if_else() {
        let src = "
            if {{ x }}
            else
            x
        ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert!(expressions.is_empty());
    }

    #[test]
    fn optimise_empty_if_else_if() {
        let src = "
            if {{ x }}
            else if {{ x }}
            else
            x
        ";
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 0 });
        assert!(expressions.is_empty());
    }

    #[test]
    fn texts() {
        let src = r#"
            text [a: b] ""
                span ""
        "#;
        let mut expressions = parse(src);
        assert_eq!(expressions.remove(0), Expression::Node { ident: 0, scope_size: 2 });
        assert_eq!(expressions.remove(0), Expression::LoadAttribute { key: 1, value: 0 });
        assert_eq!(expressions.remove(0), Expression::LoadText(0));
        assert_eq!(expressions.remove(0), Expression::Node { ident: 2, scope_size: 0 });
        assert_eq!(expressions.remove(0), Expression::LoadText(0));
        assert!(expressions.is_empty());
    }
}