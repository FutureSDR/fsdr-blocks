use fsdr_blocks::cw::shared::CWAlphabet::*;
use fsdr_blocks::cw::shared::{char_to_bb, msg_to_cw};
use futuresdr::anyhow::Result;

// cargo nextest run test_char_to_bb --no-capture
#[test]
fn test_char_to_bb() -> Result<()> {
    let mut ctbb = char_to_bb(1);

    let s = ctbb(&'S'); // Dots
                        //println!("{:?}", s);
    assert_eq!(vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0], s);

    let o = ctbb(&'O'); // Dashes
                        //println!("{:?}", o);
    assert_eq!(
        vec![1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 0.0],
        o
    );

    let wordspace = ctbb(&' '); // Wordspace
                                //println!("{:?}", wordspace);
    assert_eq!(vec![0.0, 0.0, 0.0, 0.0], wordspace);

    let underscore = ctbb(&'_'); // _ is Unknown
                                 //println!("{:?}", underscore);
    assert_eq!(vec![0.0, 0.0, 0.0, 0.0, 0.0], underscore);

    // No testcase for Letterspace in char -> Should panic

    Ok(())
}

#[test]
fn test_msg_to_cw() -> Result<()> {
    let message = "S O__S".to_uppercase().chars().collect::<Vec<char>>();
    let cw = msg_to_cw(message.as_slice());
    //println!("CW-Alphabet Vector Length: {}, Content: {:?}", cw.len(), cw);

    assert_eq!(
        vec![
            Dot,
            Dot,
            Dot,
            LetterSpace,
            WordSpace,
            Dash,
            Dash,
            Dash,
            LetterSpace,
            Unknown,
            Unknown,
            Dot,
            Dot,
            Dot,
            LetterSpace
        ],
        cw
    );
    Ok(())
}

#[test]
fn test_display_trait_impl() {
    let testdata = vec![Dash, Dot, LetterSpace, Unknown, Dot, WordSpace, Dash];
    let str: String = testdata.iter().map(ToString::to_string).collect();
    assert_eq!("-.  <?> ./ -", str)
}
