pub mod engine {
    use sqlparser::ast::*;
    use sqlparser::dialect::GenericDialect;
    use sqlparser::parser::Parser;
    use std::fmt;

    #[derive(Debug)]
    pub enum CalcResult {
        Num(f64),
        Bool(bool),
        Str(String),
    }

    impl fmt::Display for CalcResult {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                CalcResult::Num(num) => write!(f, "Result: {}", num),
                CalcResult::Bool(boolean) => write!(f, "Result: {}", boolean),
                CalcResult::Str(str) => write!(f, "Result: {}", str),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq)]
    pub enum CalcError {
        InvalidType(String),
        UnsupportedOperator(String),
        UnsupportedFunc(String),
        InvalidRequestFormat(String),
        Unexpected,
    }

    impl fmt::Display for CalcError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                CalcError::InvalidType(str) => write!(f, "[Invalid Type]: {}", str),
                CalcError::UnsupportedOperator(str) => write!(f, "[Unsupported Operator]: {}", str),
                CalcError::UnsupportedFunc(str) => write!(f, "[Unsupported Function]: {}", str),
                CalcError::InvalidRequestFormat(str) => {
                    write!(f, "[Invalid Request Format]: {}", str)
                }
                _ => write!(f, "[Unexpected Error]: Something went wrong"),
            }
        }
    }

    fn apply(
        operator: BinaryOperator,
        first_val: f64,
        second_val: f64,
    ) -> Result<CalcResult, CalcError> {
        match &operator {
            BinaryOperator::Plus => Ok(CalcResult::Num(first_val + second_val)),
            BinaryOperator::Minus => Ok(CalcResult::Num(first_val - second_val)),
            BinaryOperator::Multiply => Ok(CalcResult::Num(first_val * second_val)),
            BinaryOperator::Gt => Ok(CalcResult::Bool(first_val > second_val)),
            _ => Err(CalcError::UnsupportedOperator(String::from(
                "You try to use unsupported operator",
            ))),
        }
    }

    fn calc_binary_operation(
        left: Box<Expr>,
        op: BinaryOperator,
        right: Box<Expr>,
    ) -> Result<CalcResult, CalcError> {
        let parse_num = |expr: Expr| -> Result<f64, CalcError> {
            match calc(expr) {
                Ok(CalcResult::Num(res)) => Ok(res),
                Err(e) => Err(e),
                _ => Err(CalcError::InvalidType(String::from(
                    "Binary operators supported by Numbers only",
                ))),
            }
        };

        let (numbers, errors): (Vec<_>, Vec<_>) = [left, right]
            .into_iter()
            .map(|v| parse_num(*v))
            .partition(Result::is_ok);

        let numbers: Vec<_> = numbers.into_iter().map(Result::unwrap).collect();
        if numbers.len() == 2 {
            return apply(op, numbers[0], numbers[1]);
        } else if !errors.is_empty() {
            // TODO:
            let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
            return Err(errors[0].clone());
        }

        Err(CalcError::Unexpected)
    }

    fn parse_primitive_value(value: Value) -> Result<CalcResult, CalcError> {
        match &value {
            Value::Number(number, _) => Ok(CalcResult::Num(String::from(number).parse().unwrap())),
            Value::DoubleQuotedString(string) => Ok(CalcResult::Str(string.to_string())),
            Value::SingleQuotedString(string) => Ok(CalcResult::Str(string.to_string())),
            _ => Err(CalcError::InvalidType(String::from(
                "You try to use unsupported type",
            ))),
        }
    }

    fn calc_function(func: Function) -> Result<CalcResult, CalcError> {
        if func.name.0.get(0).unwrap().value == "SQRT" {
            let arg = func.args.first();
            if arg.is_none() {
                return Err(CalcError::InvalidType(String::from(
                    "SQRT must has an argument",
                )));
            }

            let result = {
                match &arg.unwrap() {
                    FunctionArg::Named { name: _, arg } => calc(arg.clone()),
                    FunctionArg::Unnamed(arg) => calc(arg.clone()),
                }
            };

            match &result {
                Err(e) => return Err(e.clone()),
                Ok(CalcResult::Num(num)) => return Ok(CalcResult::Num(num.sqrt())),
                _ => {
                    return Err(CalcError::InvalidType(String::from(
                        "SQRT supports only Number",
                    )))
                }
            }
        }
        Err(CalcError::UnsupportedFunc(String::from(
            "Only SQRT func is supported",
        )))
    }

    fn cast(expr: Expr) -> Result<CalcResult, CalcError> {
        match calc(expr) {
            Ok(CalcResult::Str(res)) => match res.parse::<f64>() {
                Ok(res) => Ok(CalcResult::Num(res)),
                Err(_) => Err(CalcError::InvalidType(String::from(
                    "CAST supports only Number",
                ))),
            },
            Err(e) => Err(e),
            _ => Err(CalcError::Unexpected),
        }
    }

    fn calc(expr: Expr) -> Result<CalcResult, CalcError> {
        match expr {
            Expr::BinaryOp { left, op, right } => calc_binary_operation(left, op, right),
            Expr::Function(func) => calc_function(func),
            Expr::Value(value) => parse_primitive_value(value),
            Expr::Nested(expr) => calc(*expr),
            Expr::Cast {
                expr,
                data_type: DataType::Int(_),
            } => cast(*expr),
            _ => Err(CalcError::Unexpected),
        }
    }

    pub fn exec(query: String) -> Result<CalcResult, CalcError> {
        let dialect = GenericDialect {};
        let res = Parser::parse_sql(&dialect, &query);

        if res.is_err() {
            return Err(CalcError::InvalidRequestFormat(String::from(
                "It is not SQL, man",
            )));
        }

        let ast = res.unwrap();

        if ast.is_empty() {
            return Err(CalcError::InvalidRequestFormat(String::from(
                "It is not SQL, man",
            )));
        }

        match &ast[0] {
            Statement::Query(query) => match &query.body {
                SetExpr::Select(select) => {
                    let projection = select.projection.first();
                    if projection.is_none() {
                        return Err(CalcError::InvalidRequestFormat(String::from(
                            "only SELECT is supported",
                        )));
                    }

                    match projection.unwrap() {
                        SelectItem::UnnamedExpr(expr) => calc(expr.clone()),
                        _ => Err(CalcError::InvalidRequestFormat(String::from(
                            "only Unnamed expressions are supported",
                        ))),
                    }
                }
                _ => Err(CalcError::InvalidRequestFormat(String::from(
                    "only SELECT is supported",
                ))),
            },
            _ => Err(CalcError::InvalidRequestFormat(String::from(
                "only Queries are supported",
            ))),
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn apply_operator_plus() {
            let res = apply(BinaryOperator::Plus, 1.5, 2.5);
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, 4.0);
            } else {
                panic!();
            }
        }

        #[test]
        fn apply_operator_minus() {
            let res = apply(BinaryOperator::Minus, 1.0, 2.0);
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, -1.0);
            } else {
                panic!();
            }
        }

        #[test]
        fn apply_operator_gt() {
            let res = apply(BinaryOperator::Gt, 2.0, 1.0);
            if let CalcResult::Bool(val) = res.unwrap_or(CalcResult::Bool(false)) {
                assert!(val);
            } else {
                panic!();
            }
        }

        #[test]
        fn apply_operator_multiply() {
            let res = apply(BinaryOperator::Multiply, 3.0, 2.0);
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, 6.0);
            } else {
                panic!();
            }
        }

        #[test]
        fn apply_operator_error() {
            if std::mem::discriminant(&CalcError::UnsupportedOperator(String::from("")))
                != std::mem::discriminant(&apply(BinaryOperator::GtEq, 3.0, 2.0).unwrap_err())
            {
                panic!();
            }
        }

        #[test]
        fn parse_primitive_value_number() {
            let res = parse_primitive_value(Value::Number(5.0.to_string(), false));
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, 5.0);
            } else {
                panic!();
            }
        }

        #[test]
        fn parse_primitive_value_str() {
            let res = parse_primitive_value(Value::SingleQuotedString(String::from("string")));
            if let CalcResult::Str(val) = res.unwrap_or_else(|_| CalcResult::Str(String::from(""))) {
                assert_eq!(val, String::from("string"));
            } else {
                panic!();
            }
        }

        #[test]
        fn parse_primitive_value_unsupported() {
            if std::mem::discriminant(&CalcError::InvalidType(String::from("")))
                != std::mem::discriminant(&parse_primitive_value(Value::Boolean(false)).unwrap_err())
            {
                panic!();
            }
        }

        #[test]
        fn exec_single_operator() {
            let res = exec(String::from("SELECT 1 + 1"));
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, 2.0);
            } else {
                panic!();
            }
        }

        #[test]
        fn exec_multy_operator() {
            let res = exec(String::from("SELECT 1 + 1 * 3"));
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, 4.0);
            } else {
                panic!();
            }
        }

        #[test]
        fn exec_gt_operator() {
            let res = exec(String::from("SELECT 2 > 3"));
            if let CalcResult::Bool(val) = res.unwrap_or(CalcResult::Bool(true)) {
                assert!(!val);
            } else {
                panic!();
            }
        }

        #[test]
        fn exec_operators_with_quotas() {
            let res = exec(String::from("SELECT (1 + (2+3+4)-5)+(6+7)"));
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, 18.0);
            } else {
                panic!();
            }
        }

        #[test]
        fn exec_unsupported_operators() {
            if std::mem::discriminant(&CalcError::UnsupportedOperator(String::from("")))
                != std::mem::discriminant(&exec(String::from("SELECT 1 / 2")).unwrap_err())
            {
                panic!();
            }
        }

        #[test]
        fn exec_func_sqrt() {
            let res = exec(String::from("SELECT SQRT(5 + 2 * 4)"));
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, 3.605551275463989);
            } else {
                panic!();
            }
        }

        #[test]
        fn exec_func_sqrt_gt() {
            let res = exec(String::from("SELECT SQRT(16) > SQRT(4)"));
            if let CalcResult::Bool(val) = res.unwrap_or(CalcResult::Bool(false)) {
                assert!(val);
            } else {
                panic!();
            }
        }

        #[test]
        fn exec_func_unsupported() {
            if std::mem::discriminant(&CalcError::UnsupportedFunc(String::from("")))
                != std::mem::discriminant(&exec(String::from("SELECT Log(2)")).unwrap_err())
            {
                panic!();
            }
        }

        #[test]
        fn exec_cast() {
            let res = exec(String::from("SELECT CAST('2' as int)"));
            if let CalcResult::Num(val) = res.unwrap_or(CalcResult::Num(-1.0)) {
                assert_eq!(val, 2.0);
            } else {
                panic!();
            }
        }

        #[test]
        fn exec_cast_invalied() {
            if std::mem::discriminant(&CalcError::InvalidType(String::from("")))
                != std::mem::discriminant(&exec(String::from("SELECT CAST('qwe' as int)")).unwrap_err())
            {
                panic!();
            }
        }

        #[test]
        fn exec_invalied_expr() {
            if std::mem::discriminant(&CalcError::InvalidRequestFormat(String::from("")))
                != std::mem::discriminant(&exec(String::from("SELECT * from table")).unwrap_err())
            {
                panic!();
            }
        }

        #[test]
        fn exec_invalied_statement() {
            if std::mem::discriminant(&CalcError::InvalidRequestFormat(String::from("")))
                != std::mem::discriminant(&exec(String::from("INSERT INTO table")).unwrap_err())
            {
                panic!();
            }
        }

        #[test]
        fn exec_invalied_format() {
            if std::mem::discriminant(&CalcError::InvalidRequestFormat(String::from("")))
                != std::mem::discriminant(&exec(String::from("Give the data")).unwrap_err())
            {
                panic!();
            }
        }
    }
}
