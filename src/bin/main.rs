use nananiji_calculator::ExpressionGenerator;
use std::{fs::File, path::Path};
use std::io::{Write, Read};
use anyhow::Result;
use clap::{App, Arg, arg_enum, crate_authors, crate_description, crate_name, crate_version, value_t};

arg_enum! {
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub enum ListName {
        Nananiji,
        Hanshin,
        Kyojin,
    }
}


fn main() -> Result<()> {
    env_logger::init();

    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("list_name")
            .help("The name of number set")
            .short("l")
            .long("list-name")
            .takes_value(true)
            .value_name("LIST_NAME")
            .possible_values(&ListName::variants())
            .case_insensitive(true)
            .default_value("Nananiji"))
        .arg(Arg::with_name("out_file")
            .help("Path of the directory for saving pre-calculation file")
            .short("w")
            .long("write-file")
            .takes_value(false))
        .arg(Arg::with_name("in_file")
            .help("Path of the directory for loading pre-calculation file")
            .short("r")
            .long("read-file")
            .takes_value(false))
        .arg(Arg::with_name("search_depth")
            .help("The depth of search")
            .short("d")
            .long("search-depth")
            .value_name("DEPTH")
            .default_value("3")
            .takes_value(true))
        .arg(Arg::with_name("denom_cut")
            .help("The depth of search")
            .short("c")
            .long("denom-cut")
            .value_name("MAX_DENOM")
            .default_value("10")
            .takes_value(true))
        .arg(Arg::with_name("allow_split")
            .help("Allow spliting like 3-34. This option is available for hanshin and kyojin (not nananiji)")
            .short("a")
            .long("allow-split")
            .value_name("ALLOW_SPLIT")
            .takes_value(false))
        .arg(Arg::with_name("target_num")
            .help("The number searched")
            .value_name("TARGET_NUM"))
        .get_matches();

    let list_name = value_t!(matches, "list_name", ListName)?;
    let allow_split = matches.is_present("allow_split");

    let expr_generator: ExpressionGenerator = if matches.is_present("in_file") {
        let in_filepath = file_path(list_name, allow_split);
        load_generator(in_filepath)?
    } else {
        let depth = value_t!(matches, "search_depth", usize)?;
        let denom_cut = value_t!(matches, "denom_cut", i64)?;

        match &list_name {
            &ListName::Nananiji => ExpressionGenerator::new_nananiji(depth, denom_cut),
            &ListName::Hanshin =>  ExpressionGenerator::new_hanshin(allow_split, depth, denom_cut),
            &ListName::Kyojin => ExpressionGenerator::new_kyojin(allow_split, depth, denom_cut),
        }
    };

    if matches.is_present("out_file") {
        let out_filepath = file_path(list_name, allow_split);
        save_generator(out_filepath, &expr_generator)?;
    } else if matches.is_present("target_num") {
        let target_num = value_t!(matches, "target_num", i64)?;
        println!("{} = {}", expr_generator.generate(target_num), target_num);
    } else {
        println!("{}", matches.usage());
    }


    Ok(())
}

fn file_path(list_name: ListName, allow_split: bool) -> &'static Path {
    match (list_name, allow_split) {
        (ListName::Nananiji, _)    => Path::new("nananiji.bin"),
        (ListName::Hanshin, true)  => Path::new("hanshin_a.bin"),
        (ListName::Hanshin, false) => Path::new("hanshin.bin"),
        (ListName::Kyojin, true)   => Path::new("kyojin_a.bin"),
        (ListName::Kyojin, false)  => Path::new("kyojin.bin"),
    }
}

fn load_generator(filepath: &Path) -> Result<ExpressionGenerator> {
    let mut file = File::open(filepath)?;
    let mut u8_encoded = Vec::new();
    file.read_to_end(&mut u8_encoded)?;
    let generator: ExpressionGenerator = bincode::deserialize(&u8_encoded)?;

    Ok(generator)
}

fn save_generator(filepath: &Path, generator: &ExpressionGenerator) -> Result<()> {
    let u8_encoded = bincode::serialize(&generator)?;
    let mut file = File::create(filepath)?;
    file.write_all(&u8_encoded[..])?;

    Ok(())
}
