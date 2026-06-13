fn main() {
    let code = autoskill_md::cli::run(std::env::args().skip(1).collect());
    std::process::exit(code);
}
