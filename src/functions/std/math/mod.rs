pub enum MathExpression {
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
}

impl MathExpression {
    pub fn add(a: f64, b: f64) -> f64 {
        a + b
    }

    pub fn subtract(a: f64, b: f64) -> f64 {
        a - b
    }

    pub fn multiply(a: f64, b: f64) -> f64 {
        a * b
    }

    pub fn divide(a: f64, b: f64) -> f64 {
        if b == 0.0 {
            panic!("Division by zero");
        }
        a / b
    }

    pub fn power(a: f64, b: f64) -> f64 {
        a.powf(b)
    }

    pub fn modulo(a: f64, b: f64) -> f64 {
        if b == 0.0 {
            panic!("Modulo by zero");
        }
        a % b
    }

    /// Creates a MathExpression from a string
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "std::math::add" => Some(MathExpression::Add),
            "std::math::subtract" => Some(MathExpression::Subtract),
            "std::math::multiply" => Some(MathExpression::Multiply),
            _ => None,
        }
    }

    /// Gets the function associated with this expression
    pub fn get_function(&self) -> fn(f64, f64) -> f64 {
        match self {
            MathExpression::Add => Self::add,
            MathExpression::Subtract => Self::subtract,
            MathExpression::Multiply => Self::multiply,
            MathExpression::Divide => Self::divide,
            MathExpression::Power => Self::power,
            MathExpression::Modulo => Self::modulo,
        }
    }

    /// Evaluates the math expression with the provided operands
    pub fn evaluate(&self, a: f64, b: f64) -> f64 {
        let func = self.get_function();
        func(a, b)
    }
}

// Example of how to use MathExpression:
//
// fn main() {
//     // Deserialize from string
//     let add_expr = MathExpression::from_string("add").unwrap();
//     let mul_expr = MathExpression::from_string("multiply").unwrap();
//     let pow_expr = MathExpression::from_string("power").unwrap();
//
//     // Get functions and call them
//     let add_func = add_expr.get_function();
//     let mul_func = mul_expr.get_function();
//     let pow_func = pow_expr.get_function();
//
//     let result1 = add_func(5.0, 3.0);  // 5.0 + 3.0 = 8.0
//     let result2 = mul_func(5.0, 3.0);  // 5.0 * 3.0 = 15.0
//     let result3 = pow_func(2.0, 3.0);  // 2.0^3.0 = 8.0
//
//     // Or use evaluate directly
//     let result4 = add_expr.evaluate(5.0, 3.0);  // 5.0 + 3.0 = 8.0
//
//     println!("5.0 + 3.0 = {}", result1);
//     println!("5.0 * 3.0 = {}", result2);
//     println!("2.0^3.0 = {}", result3);
//     println!("5.0 + 3.0 = {}", result4);
// }
