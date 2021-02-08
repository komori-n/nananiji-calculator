use lambda_runtime::{error::HandlerError, lambda, Context};
use nananiji_calculator::ExpressionGenerator;
use serde::{Serialize, Deserialize};
use std::{fs::File, path::Path};
use std::io::Read;
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(tag = "name", content = "split", rename_all="lowercase")]
enum ListName {
    Nananiji,
    Hanshin(bool),
    Kyojin(bool),
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Request {
    value: String,
    list_name: ListName,
}

#[derive(Serialize, Clone)]
struct RequestResult {
    req: Request,
    expr: String,
}

struct PreloadGenerators {
    nananiji: ExpressionGenerator,
    hanshin: ExpressionGenerator,
    hanshin_a: ExpressionGenerator,
    kyojin: ExpressionGenerator,
    kyojin_a: ExpressionGenerator,
}

impl PreloadGenerators {
    fn choose_generator(&self, list_name: ListName) -> &ExpressionGenerator {
        match list_name {
            ListName::Nananiji       => &self.nananiji,
            ListName::Hanshin(true)  => &self.hanshin_a,
            ListName::Hanshin(false) => &self.hanshin,
            ListName::Kyojin(true)   => &self.kyojin_a,
            ListName::Kyojin(false)  => &self.kyojin,
        }
    }
}

fn main() -> Result<()> {
    let gens = PreloadGenerators {
        nananiji: load_generator(Path::new("nananiji.bin"))?,
        hanshin: load_generator(Path::new("hanshin.bin"))?,
        hanshin_a: load_generator(Path::new("hanshin_a.bin"))?,
        kyojin: load_generator(Path::new("kyojin.bin"))?,
        kyojin_a: load_generator(Path::new("kyojin_a.bin"))?,
    };

    lambda!(move |req: Request, _ctx: Context| handler(req, &gens));

    Ok(())
}

fn handler(req: Request, gens: &PreloadGenerators) -> Result<RequestResult, HandlerError> {
    println!("{:?}", req);

    let generator = gens.choose_generator(req.list_name);
    let value = req.value.parse::<i64>()
        .map_err(|_| HandlerError::from("value parse failed"))?;
    let expr = generator.generate(value);

    Ok(RequestResult {
        req,
        expr
    })
}

fn load_generator(filepath: &Path) -> Result<ExpressionGenerator> {
    let mut file = File::open(filepath)?;
    let mut u8_encoded = Vec::new();
    file.read_to_end(&mut u8_encoded)?;
    let generator: ExpressionGenerator = bincode::deserialize(&u8_encoded)?;

    Ok(generator)
}