pub mod lexer;
pub mod parser;

pub fn test_parser_function() {
    let input = r#"@badge-info=subscriber/8;badges=vip/1,subscriber/6,twitch-recap-2023/1;color=#FFBEDF;display-name=plss;emote-sets=0,793,8231,19194,876326,935873,1232221,300374282,300380967,301464307,302029931,302512232,302792003,303148195,323827706,326691955,334292379,344011590,345474279,366226437,387726113,390658648,392630734,409842248,415514593,416564655,418871744,427477847,435300334,440880357,441442142,454806117,459526139,460760505,468360508,470888728,477339272,484906151,496680382,537206155,1306162089,1911289880,15a031d7-8783-468d-99f2-f5832a08d7c0,35b067de-37af-4430-99b0-6591201aa8c7,398cca87-aea0-4fd7-b29d-0613ab67320a,3c5be0d3-3eb7-4e96-93e2-44ac38b40819,5263b216-dab4-47e5-bc72-94fa093f6906,560c6a32-134b-4340-8185-a3e99e87237b,7c63ed2d-8e7e-4525-85a4-51e0b78ad0e3,7d68dda4-5170-442a-8dd8-9e5eb1ed8d27,acccd20c-25a2-497f-8265-59b890b61d62,bc112c6f-a202-43c2-b144-2c93e20cc5a2,bd70e005-1bb7-4879-b910-67779c22ccf9,bd70e005-1bb7-4879-b910-67779c22ccf9,c64918b8-0ebd-41c9-b153-300ca3491aa8,c9a93654-bae4-439e-ac62-0d69ecad1786,d31f1a6c-72ee-4aab-9bd3-7bf7f1d037bc,d92eb0a5-4f2b-43f6-892d-bc398567a0e1,e3ac0383-f23b-4dcf-ad65-d5a7ee1b26cb,ebe796ee-3c56-472c-922a-af70aeeff96d,ed963b8b-9b40-4d60-ba5b-f68985586441;mod=0;subscriber=1;user-type= :tmi.twitch.tv USERSTATE #sleepiebug @emote-only=0;followers-only=-1;r9k=0;room-id=610533290;slow=0;subs-only=0 :tmi.twitch.tv ROOMSTATE #sleepiebug"#;
    let parser = parser::IrcParser::new();

    let message = parser.parse_socket_data(input).unwrap();
    let chat_content = parser.get_chat(&message);

    println!("{:?}", &message);
    println!("{:?}", chat_content);
}
