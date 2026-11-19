fn main() {
    let diagnosis = media::doctor::diagnose();
    if diagnosis.ok {
        print!("{}", diagnosis.text);
    } else {
        eprint!("{}", diagnosis.text);
        std::process::exit(1);
    }
}
