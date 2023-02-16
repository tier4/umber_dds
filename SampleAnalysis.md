# SampleAnalysis
RustDDSを参考実装としてRTPSの解析を行う

## Subscriberの解析
RustDDSに付属のShapeDemoをSubscriberとして動作させてRTPSのSubscriber側の解析をおこなう

### main.rs
- line 65 - 68
新しく20個のスレッドが生成されることを確認。
このタイミングでUDPマルチキャストのグループに加入したことを知らせるIGMPv3のパケットをキャプチャ
```
$ lsof
shapes_de 160 root    3u  IPv4 450292      0t0  UDP *:7400
shapes_de 160 root    4u  IPv4 450294      0t0  UDP *:7410
shapes_de 160 root    5u  IPv4 450295      0t0  UDP *:7401
shapes_de 160 root    6u  IPv4 450297      0t0  UDP *:7411
shapes_de 160 root   10u  IPv4 457163      0t0  UDP *:35442
shapes_de 160 root   11u  IPv4 457165      0t0  UDP 69d7d3c3fefa:39541 // このポートは毎回変わる
                                                                        // このポート番号を以下rpとする
```
`DomainParticipant::new(domain_id)`(src/dds/participant.rs)が実行
大まかな構造
```
DomainParticipant::new() {
    DomainParticipantDisc::new()
    Discovery::new()
}
DomainParticipantDisc::new() {
    DomainParticipantInner::new()
}
DomainParticipantInner::new() {
    UDPListener::new_multicast()
    // "0.0.0.0:domain_id"を"239.255.0.1"のマルチキャストグループに加入
    // この他にも4つUDPListener::new_multicast()がある
}
```



- line 68 - 92
特にパケットは送信されない
- line 92 - 93
このとき20個のスレッドが起動されているから、パケットがどのスレッドから送信されたか注意
rpポートからマルチキャスト:7400にRTPSパケットを5つ送信
RTPS Submessage (p. 44)
    The Entity Submessage
        HEARTBEAT Submessage: Writerが1つ以上のReaderに向けてWriterの持っている情報を説明する
        Data: ReaderまたはWriterによって送られる、application Data-objectの値に関する情報を含む。
    The Interpreter Submessage
        InfoTimestamp: 次のEntity Submessageのsource timestampを提供する

パケットの詳細
最初の4つは長さ106のINFO_TS, HEARTBEAT
最後の1つは長さ310のDATA(p), HEARTBEAT
5つのパケットで共通
    Protocol version 2.4
    venderId 01.18 (Unknown) ; 01.18はプロトコルによって予約された値(p. 20)
    guidPrefix ~省略(12 octet)~ ; 同一のparticipantであれば一致する値(p. 19)
長さ106のINFO_TS, HEARTBEAT
    INFO_TS
        Flags: 0x01 ; Endianness bit set
        Timestamp ; 時刻
        octetsToNextHeader: 8
    HEARBEAT
        Flags: 0x01 ; Endianness bit set
        octetsToNextHeader: 28
        ReaderEntityId
            Kye: 0x000000
            Kind: Application-defined unknown king 0x00
        writerEntityId
            Key: (0x4, 0x3, 0x200, 0x2) ;  (パケット1, パケット２, パケット3, パケット4)
            Kind: 0xc2; Build-in writer (with key)
        firstAvailableSeqNumber: 1
        lastSeqNumber: 0
        count: 2
長さ310のDATA(p), HEARTBEAT
    DATA
        Flags: 0x5 ; (Data present, Endianness bit) set
        OctetsToNextHeader: 212
        Extra Flags: 0x0
        Octets to inline QoS: 16
        ReaderEntityId
            Kye: 0x000000
            Kind: Application-defined unknown king 0x00
        writerEntityId
            Key: 0x100
            Kind: 0xc2; Build-in writer (with key)
        writerSeqNumber: 1
        serializedData
            encapsulation kind: PL_CDR_LE 0x3
            encapsulation options: 0x0
            serializedData
                PID_~~
                ~~
                PID_~~


    HEARBEAT
        Flags: 0x01 ; Endianness bit set
        octetsToNextHeader: 28
        ReaderEntityId
            Key: 0x000000
            Kind: Application-defined unknown king 0x00
        writerEntityId
            Key: 0x100
            Kind: 0xc2; Build-in writer (with key)
        firstAvailableSeqNumber: 1
        lastSeqNumber: 1
        count: 2

最初の4つの長さ106のINFO_TS, HEARTBEATのrtpsパケットを送信してる箇所
thread 2 "RustDDS Partici"の
dds/dp_event_loop.rs:229: match EntityId::from_token(event.token()) { ; この時点ではキャプチャされない
dds/dp_event_loop.rs:316: TokenDecode::Entity(eid) => { : EntityId ; ここに到達した時点で1つキャプチャされる
2回目229行目に到達した時点で2つめをキャプチャ
3回目229行目に到達した時点で3つめをキャプチャ
4回目229行目に到達した時点で4つめをキャプチャ
TODO: このパケットを送信してるコードを見つけ出す



