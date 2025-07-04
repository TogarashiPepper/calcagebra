use std::{
	collections::HashMap,
	f32::consts::{E, PI},
	ops::Range,
};

pub type InterpreterContext<'a> = (
	&'a mut HashMap<String, Data>,
	&'a mut HashMap<String, Function>,
);

use crate::{
	errors::Error,
	expr::Expression,
	standardlibrary::{io, math, operators},
	types::{Data, DataType},
};

#[derive(Debug, Clone)]
pub struct Interpreter {
	pub globals: HashMap<String, Data>,
	pub functions: HashMap<String, Function>,
}

impl Default for Interpreter {
	fn default() -> Self {
		Self::new()
	}
}

impl Interpreter {
	pub fn new() -> Self {
		let mut globals = HashMap::new();
		let mut functions = HashMap::new();

		let sys_vrs = [
			("i", Data::Number(0.0, 1.0)),
			("pi", Data::Number(PI, 0.0)),
			("π", Data::Number(PI, 0.0)),
			("e", Data::Number(E, 0.0)),
		];

		for (name, val) in sys_vrs {
			globals.insert(name.to_string(), val);
		}

		[
			"print",
			"read",
			"int",
			"real",
			"add",
			"sub",
			"mul",
			"div",
			"pow",
			"rem",
			"is_eq",
			"neq",
			"gt",
			"gteq",
			"lt",
			"lteq",
			"abs",
			"round",
			"ceil",
			"floor",
			"ln",
			"log10",
			"log",
			"sin",
			"cos",
			"tan",
			"sqrt",
			"cbrt",
			"nrt",
			"graph",
			"transpose",
			"determinant",
			"adj",
			"inverse",
		]
		.map(|name| {
			functions.insert(
				name.to_string(),
				Function::STD(STDFunction {
					name: name.to_string(),
				}),
			)
		});

		Self { globals, functions }
	}

	pub fn interpret(&mut self, ast: Vec<(Expression, Range<usize>)>) -> Result<(), Error> {
		let ctx = &mut (&mut self.globals, &mut self.functions);

		for (expr, range) in ast {
			expr.evaluate(ctx, range)?;
		}

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub enum Function {
	UserDefined(UserDefinedFunction),
	STD(STDFunction),
}

#[derive(Debug, Clone)]
pub struct UserDefinedFunction {
	pub params: Vec<(String, DataType)>,
	pub return_type: DataType,
	pub code: Expression,
	pub range: Range<usize>,
}

impl UserDefinedFunction {
	pub fn execute<'a, 'b>(self, ctx: &'a mut InterpreterContext<'b>) -> Result<Data, Error>
	where
		'b: 'a,
	{
		self.code.evaluate(ctx, self.range)
	}
}

#[derive(Debug, Clone)]
pub struct STDFunction {
	pub name: String,
}

impl STDFunction {
	pub fn execute<'a>(
		self,
		ctx: &'a mut InterpreterContext<'a>,
		exprs: Vec<(Expression, Range<usize>)>,
	) -> Result<Data, Error> {
		if &self.name == "graph" {
			return math::graph(&exprs[0].0, ctx);
		}

		let mut args = vec![];

		for (expr, range) in exprs {
			let data = expr.evaluate(ctx, range)?;
			args.push(data);
		}

		let data = match self.name.as_str() {
			"print" => io::print(args),
			"read" => io::read(ctx)?,
			"add" => operators::add(&args[0], &args[1]),
			"sub" => operators::sub(&args[0], &args[1]),
			"mul" => operators::mul(&args[0], &args[1]),
			"div" => operators::div(&args[0], &args[1]),
			"pow" => operators::pow(&args[0], &args[1]),
			"rem" => operators::rem(&args[0], &args[1]),
			"is_eq" => operators::is_eq(&args[0], &args[1]),
			"neq" => operators::neq(&args[0], &args[1]),
			"gt" => operators::gt(&args[0], &args[1]),
			"gteq" => operators::gteq(&args[0], &args[1]),
			"lt" => operators::lt(&args[0], &args[1]),
			"lteq" => operators::lteq(&args[0], &args[1]),
			"abs" => math::abs(&args[0]),
			"round" => math::round(&args[0]),
			"ceil" => math::ceil(&args[0]),
			"floor" => math::floor(&args[0]),
			"ln" => math::ln(&args[0]),
			"log10" => math::log10(&args[0]),
			"log" => math::log(&args[0], &args[1]),
			"sin" => math::sin(&args[0]),
			"cos" => math::cos(&args[0]),
			"tan" => math::tan(&args[0]),
			"sqrt" => math::sqrt(&args[0]),
			"nrt" => math::nrt(&args[0], &args[1]),
			"transpose" => math::transpose(&args[0]),
			"determinant" => math::determinant(&args[0]),
			"adj" => math::adj(&args[0]),
			"inverse" => math::inverse(&args[0]),
			_ => unreachable!(),
		};

		Ok(data)
	}
}
