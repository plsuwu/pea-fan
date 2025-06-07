use super::base::InputStream;

pub fn test_parser() {
    let test_string = "@badge-info=;badges=broadcaster/1,twitch-recap-2023/1;client-nonce=b5e3cf09c8800345fdd49e8fac6e7c00;color=#FFBEDF;display-name=plss;emotes=;first-msg=0;flags=;id=77ac96fb-34c4-4494-b4a2-83873aecb333;mod=0;returning-chatter=0;room-id=103033809;subscriber=0;tmi-sent-ts=1749208156695;turbo=0;user-id=103033809;user-type= :plss!plss@plss.tmi.twitch.tv PRIVMSG #plss :eeeeeeeee\r\n";
    println!("\n{:?}\n\n", test_string);

    let mut parser = InputStream::new(test_string);
}
