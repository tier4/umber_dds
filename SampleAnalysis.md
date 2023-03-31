# SampleAnalysis
RustDDSを参考実装としてRTPSの解析を行う

## Subscriberの解析
RustDDSに付属のShapeDemoをSubscriberとして動作させてRTPSのSubscriber側の解析をおこなう

### main.rs
- line 65 - 65
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
開かれるportはp. 165にある。RustDDSのポートを決める実装はsrc/network/constant.rsにあり、仕様書のdefault通りに実装されている。
上4つは受信用, 下2つは送信用。
受信用はsubscribeのため、送信用はdiscoveryのため。

line 65 `DomainParticipant::new(domain_id)`(src/dds/participant.rs)を実行
dds/participant.rsの大まかな構造
```
DomainParticipant::new() {
    DomainParticipantDisc::new()
    Discovery::new()
}
DomainParticipantDisc::new() {
    DomainParticipantInner::new()
}
DomainParticipantInner::new() {

    // Discovery trafficのためのunicast, multicastのソケットをopen
    UDPListener::new_multicast()
    // "0.0.0.0:spdp_well_known_multicast_port(domain_id)"を開いて"239.255.0.1"のマルチキャストグループに加入
    UDPListener::new_unicast()
    // "0.0.0.0:spdp_well_known_unicast_port(domain_id, , participant_id)"を開く

    // User trafficのためのunicast, multicastのソケットをopen
    UDPListener::new_multicast()
    // "0.0.0.0:user_traffic_multicast_port(domain_id)"を開いて"239.255.0.1"のマルチキャストグループに加入
    UDPListener::new_unicast()
    // "0.0.0.0:user_traffic_unicast_port(domain_id, participant_id)"を開く

    let ev_loop_handle = thread::Builder::new()
        .spawn(move || {
            let dp_event_loop = DPEventLoop::new()
            dp_event_loop.event_loop();
            // ここでUDP *:rpがopenされる
        })
}
```
dds/dp_event_loop.rs
```
DPEventLoop::new() {
    // こいつがDomainParticipantInner::new()で開いたsocketとTokenのhashmapを受け取り所有する
    // port number 0 means OS chooses an available port number.
    // ポート番号0はOSが使用可能なポート番号を選択することを意味する。
    let udp_sender = UDPSender::new(0).expect("UDPSender construction fail");
}

```

network/udp_sender.rs
```
// 構造体UDPSender を定義している理由はおそらく、LocatorListを受け取ってそこにデータを送信するためには、複数のSocketを１つの構造体にまとめて管理しておいたほうが便利だから。
UDPSender::new() {
    let unicast_socket = {
        let saddr: SocketAddr = SocketAddr::new("0.0.0.0".parse().unwrap(), sender_port);
        UdpSocket::bind(&saddr)?
        // UDP *:35442をオープン
    };
    // We set multicasting loop on so that we can hear other DomainParticipant
    // instances running on the same host.
    // unicast_socketがunicastでのデータの送信だけに使うのであれば、multicast_loop_v4をtrueにする必要はないのでは？
    // multicastの受信に使うの？
    // ここではUdpSocketがmio::UdpSocketになってるのはなんで？
    // net::UdpSocketで良くないの？
    unicast_socket UdpSocket
        .set_multicast_loop_v4(true)

    let mut multicast_sockets = Vec::with_capacity(1);
    for multicast_if_ipaddr in get_local_multicast_ip_addrs()? {
        // 69d7d3c3fefa:39541をオープン
    }
}
```

- line 65 - 92
特にパケットは送信されない


- line 92 - 93

このとき20個のスレッドが起動されているから、パケットがどのスレッドから送信されたか注意

rpポートからマルチキャスト:7400にRTPSパケットを5つ送信
```
RTPS Submessage (p. 44)
    The Entity Submessage
        HEARTBEAT Submessage: Writerが1つ以上のReaderに向けてWriterの持っている情報を説明する
        Data: ReaderまたはWriterによって送られる、application Data-objectの値に関する情報を含む。
    The Interpreter Submessage
        InfoTimestamp: 次のEntity Submessageのsource timestampを提供する
```
```
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
        // amd64はlittele endiannだから0x01になってると思われる
        Timestamp ; 時刻
        // InvalidateFlag がヘッダーにないときのみ使われる
        // 次のSubmessageを処理するために使われるtimestamp (p. 59)
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
        // このserializedDataはリモートParticipantを探すのためのSPDPdiscoveredParticipantData (p. 118)

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
```
最初の4つの長さ106のINFO_TS, HEARTBEATのrtpsパケットを送信してる箇所

thread 2 "RustDDS Partici"の
`dds/dp_event_loop.rs:229: match EntityId::from_token(event.token()) { ; この時点ではキャプチャされない`
`dds/dp_event_loop.rs:316: TokenDecode::Entity(eid) => { : EntityId ; ここに到達した時点で1つキャプチャされる`

2回目229行目に到達した時点で2つめをキャプチャ

3回目229行目に到達した時点で3つめをキャプチャ

4回目229行目に到達した時点で4つめをキャプチャ

TODO: このパケットを送信してるコードを見つけ出す
-> 多分writer.process_writer_command()とか、ev_wrapper.message_receiver.handle_received_packet(&packet);の中で送信されてる

名前からsend_to_udp_socketでパケットを送信してると思われるから、これにbreakポイント貼って調査

## DomainParticipantの構造
```
struct DomainParticipant {
    dpi: Arc<Mutex<DomainParticipantDisc>>,
}

struct DomainParticipantDisc {
    dpi: Arc<Mutex<DomainParticipantInner>>,
    // Discovery control
    discovery_command_sender: mio_channel::SyncSender<DiscoveryCommand>,
    discovery_join_handle: mio_channel::Receiver<JoinHandle<()>>,
    // This allows deterministic generation of EntityIds for DataReader, DataWriter, etc.
    // EntitiyIdを決定するためのもの？
    entity_id_generator: atomic::AtomicU32,

}

pub struct DomainParticipantInner {
    domain_id: u16,
    participant_id: u16,

    my_guid: GUID,

    // Adding Readers
    sender_add_reader: mio_channel::SyncSender<ReaderIngredients>,
    sender_remove_reader: mio_channel::SyncSender<GUID>,

    // dp_event_loop control
    stop_poll_sender: mio_channel::Sender<()>,
    ev_loop_handle: Option<JoinHandle<()>>, // this is Option, because it needs to be extracted
    // out of the struct (take) in order to .join() on the handle.

    // Writers
    add_writer_sender: mio_channel::SyncSender<WriterIngredients>,
    remove_writer_sender: mio_channel::SyncSender<GUID>,

    dds_cache: Arc<RwLock<DDSCache>>,
    discovery_db: Arc<RwLock<DiscoveryDB>>,
    discovery_db_event_receiver: mio_channel::Receiver<()>,

    // RTPS locators describing how to reach this DP
    self_locators: HashMap<Token, Vec<Locator>>,
}
```
openしたsocketはev_loop_handleが持つ。


DomainParticipantInnerがGUID(globally Unique Id)をもってる。

"The GUID (Globally Unique Identifier) is an attribute of all RTPS Entities and uniquely identifies the Entity within a DDS Domain" (p. 24)

## DPEvnetLoopの調査
`HashMap<mio::Token, UdpListener>`を受け取り、mioのpollにUdpListenerを登録

loopの中でpoll.pool()、イベントが発火したらそれを処理。
イベントが`DISCOVERY_*`の場合受信内容を`UDPListener::messages`でVec<Byte>にして、forでVecの要素を順に`MessageReceiver::handle_received_packet()`に渡して処理する。

~~受け取った内容をどうやってb'RTPS'から始まるパケットに分割してるか不明。~~
socket::receive()で受け取れるのが、b'RTPS'から始まるパケット1つ。
ループの中でsocket::receive()を実行して、逐次Vecにpushしてる。

## UdpListener::messages()の調査
socket::receive()して、4byteにアラインメントされてるか確認(必要？)
アラインメントされてなければ0xCCを追加(0xCCの根拠はない-> アクセスされないから)
送信時にアラインメントされないの？
TODO: なんかよくわかんない処理が行われているから必要かどうか調査する。

### ByteMutの取扱
buf = BytesMut::with_capacity(CAP)を実行するとスタックフレーム上に[*buf, CAP, len(=0)]が作られる。ことのき、ヒープ上のbufは最大でCAP byte確保できるが、現在確保されているサイズはlen byte。
lenを手動でセットするにはunsafeの中である必要がある。(CAP >= lenをプログラマが保証すれば安全にセットできる)
bufのlenを設定せずにlister.recev(&buf)すると、bufの長さが0になり何も受け取れない。

### Bloking IO と Non-Bloking IO
- Bloking IO

パケットが受信できるまで待機する。UDPListenerのデフォルト。

- Non-Bloking IO

パケットが受信できない場合、エラーを返す。`.set_nonblocking(true)`


## MessageReceiverの調査
dds/message_receiver.rs

MessageReceiverは仕様書 8.3.4: The RTPS Message Receiver で説明されている、submessageの連続体を解釈するもの。submaessageの連続体をパースするためにmessage/submessageのデシリアライザーを呼ぶ。そして、Interpreter Submessageの命令を実行し、Entity Submessageのデータを適切なEntityに渡す(仕様書 8.3.7を参照)

(spec 8.3.4)Submessageの解釈と意味は同じMessageに含まれるそれより前のSubmessageに依存する。したがってMessageのreceiverは同じMessageに含まれるそれ以前にdeserializeされたSubmessageの状態を管理しなければならない。RTPS receiverの状態としてmodelされた状態は、新しいmessageが処理されたときにリセットされ、それぞれのSubmessageの解釈に文脈を提供する

/src/dds/message_receiver.rs

~~TODO:~~
MessageReceiver::new()で*_reply_locator_listの初期値が`vec![Locator::Invalid]`になっている。しかし、仕様書のp. 38にはLocatorの初期値には受信したメッセージにしたがって値をセットすると書いてあるから、RustDDSが初期値をInvalidに設定している理由を調査。

-> "The list is initialized to contain a single Locator_t with the LocatorKind,"と書いてあるから要素を1つ含むVecとして初期化しないといけない。
しかし、コンストラクターを実行するのは受信前だからアドレスもポートも設定できないからINVALID一つを要素として初期化している。

### Message Receiverが従うルール (spec 8.3.4.1)
1. full Submessage headerを読み込めない場合、残りのMessageは壊れていると考える
2. submessageLengthフィールドは次のsubmessageがどこから始まるかを定義する、もしくは、Section 8.3.3.2.3(p. 34)で示されるようにMessageの終わりを拡張するSubmessageを指し示す。もしこのフィールドが無効なら、残りのMessageは無効である。
3. 未知のSubmessageIDをもつSubmessageは無視されなければならず、次のSubmessageに継続してパースされなければならない。具体的にRTPS 2.4の実装ではversion 2.4で定義されているSubmessageKind以外のIDをもつSubmessageは無視される。
未知のvenderId由来のvender-specificの範囲のSubmessageIdも無視されなければならず、次のSubmessageに継続してパースされなければならない。
4. Submessage flags.Submessageのreceiverは未知のflagを無視されるべきである。RTPS2.4の実装では"X"(unused)とプロトコルにマークされたすべてのフラッグは飛ばされるべきである。
5. 正しいsubmessageLengthフィールドは既知のIDをもつSubmessageであっても、常に次のSubmessageを探すのに使われなくてはならない。(おそらく、既知の種類のSubmessageで長さがわかっている場合でも、versionが上がって新しくElementが追加されている可能性があるから)
6. 既知だが、無効なSubmessageは残りのMessage(the rest of the Message)を無効にする。 // "the rest of the Message"が何を指すのか仕様書から読み取れないが、RustDDSの実装は、無効なSubmessageが含まれるMessageを無効なものとして破棄している

### guid_prefix, EntityIdの調査
- guid_prefix
    先頭2 octetはvenderIdの先頭2 octetと同じにする。これによってDDS Domain内で複数のRTPS実装が使われてもguidが衝突しない。残りの 10 octetは衝突しなければどんな方法で生成してもいい。(p. 144)

### MessageReceiver::handle_received_packet()の調査
MessageReceiver::handle_received_packet()
- DDSPINGかどうか確認
    先頭4byteが"RTPS"か、先頭9から16byteが"DDSPING"か確認。
    DDSPINGだった場合何をすべきか仕様書に書いてない
- Speedy readerを呼ぶ
Crate speedy (https://docs.rs/speedy/latest/speedy/index.html)

Speedy readerはバイナリをシリアライズするためのもので、RustDDSではエンディアンを実行時に決めるために、代わりにMessageを実装している。
`let rtps_message = match Message::read_from_buffer(msg_bytes) {}`
- メッセージを処理する
`self.handle_parsed_message(rtps_message);`

### timestamp
src/structure/time.rs
```
pub struct Timestamp {
    seconds: u32,
    fraction: u32,
}
impl Timestamp {
    fn from_nanos(nanos_since_unix_epoch: u64) -> Self {
        Self {
            seconds: (nanos_since_unix_epoch / 1_000_000_000) as u32,
            fraction: (((nanos_since_unix_epoch % 1_000_000_000) << 32) / 1_000_000_000) as u32,
        }
    }
}
```
8 octet(64 bit)で上位4 octetがunix epochの秒の部分、下位4 octetがunix epochの秒より細かい部分

## Message::read_from_buffer(msg_bytes)
msg_bytesはBytes::bytes型の理由 -> enndiannの扱いが楽だからと思ったけど、ちがうかも。よくわかんない
RTPS Headerは20 Byte (p. 155)
SubmessageHeaderは4 Byte (p. 156)
- SubmessageのoctetsToNextHeaderの意味 (p. 157)
    - octetsToNextHeader == 0
        - PAD or INFO_TS

            contensの大きさは0
        - NOT (PAD or INFO_TS)

            contensの大きさは0,
            SubmessageはMessageの中で最後となり、Messageの大きさを広げる

            This makes it possible to send Submessages larger than 64k (the size that can be stored in the octetsToNextHeader field), provided they are the last Submessage in the
Message.

    - octetsToNextHeader > 0
        - SubmessageがMessageの中で最後でない場合

            submessageのcontensの最初のcotetから次のsubmessageの最初のoctetまでのoctet数
        - SubmessageがMessageの中で最後な場合

            submessageのheaderを除いたMessageの残りのoctet数

endianness_flagを取得
RTPS SubmessageはInterpreter-SubmessageとEntity-Submessageの２グループに分けられる。(p. 44)

Entity-Submessageは1つの RTPS Entityに向けたもの。
Interpreter-SubmessageはRTPS Receiverの状態を変化させ、次のEntity-Submessageの処理を助けるコンテキストを提供する。

Submessage IDごとにそれぞれ処理する

## Submessage
submessageId: 1 octet, flags: 1 octet, octetsToNexHeader: 2 octet

flagsの各ビットの意味はSubmessageの種類によって変化する

各Submessageの詳細はp. 45〜


## DomainParticipant::new(domain_id)からの実行path
```
DomainParticipant::new() -> DomainParticipantDisc::new() -> DomainParticipantInner::new()
```

## SubmessageKindがstructで定義されている理由
messages/submessages/submessage_kind.rs
```
pub struct SubmessageKind {
    value: u8,
}
```
submessageIdは0x00..=0x7fの範囲はRTPSプロトコルで予約されていて、
0x80..=0xffはベンダーが自由に使うために予約されている。
RTPS version 2.4では13種類のSubmessageKindが定義されているが、メジャーバージョン増えると増える可能性がある。
enumだとsubmessageIdがv2.4で定義された13種類以外を受信したときにそのIDを保持できないから。

### Heartbeatのflag
RTPS 2.3のspec 8.3.7.5にはHeartbeatのflagは[Endianness, Final, Liveliness, GroupInfo]の4つがあるけど、
RustDDSにはGroupInfoがなくて3つしかない。(WireSharkもGroupInfoがない)

TODO: RTPS 2.4で削除された可能性があるので調査
-> 2.4の仕様書が見つからない。2.3の次が2.5になってる。

## AckNack
Writerで使われるsequence numberに関連するReaderの状態を共有するためにReaderがWriterに送るsubmessage.

AckNackは２つの目的を同時に提供する。
- 
- 

```
// src/dds/message_receiver.rs
    EntitySubmessage::AckNack(acknack, _) => {
        // Note: This must not block, because the receiving end is the same thread,
        // i.e. blocking here is an instant deadlock.
        match self
            .acknack_sender
            // このacknack_senderの対になるreceiverはMessagereceiverを所有しているDPEventLoopが持っている
            // TODO: この辺のRTPSとDDSの関係性を調査して実装
            // DPEventLoopがacknack_senderからなにか受け取ると、handle_writer_acknack_action()で処理する
            .try_send((self.source_guid_prefix, AckSubmessage::AckNack(acknack)))
        {
            Ok(_) => (),
            Err(TrySendError::Full(_)) => {
                info!("AckNack pipe full. Looks like I am very busy. Discarding submessage.");
            }
            Err(e) => warn!("AckNack pipe fail: {:?}", e),
        }
    }
// src/dds/dp_event_loop.rs
    fn handle_writer_acknack_action(&mut self, _event: &Event) {
        while let Ok((acknack_sender_prefix, acknack_submessage)) = self.ack_nack_receiver.try_recv() {
            let writer_guid = GUID::new_with_prefix_and_id(
                self.domain_info.domain_participant_guid.prefix,
                acknack_submessage.writer_id(),
            );
            if let Some(found_writer) = self.writers.get_mut(&writer_guid.entity_id) {
                if found_writer.is_reliable() {
                    found_writer.handle_ack_nack(acknack_sender_prefix, &acknack_submessage);
                    // DPEventLoopがもってるwriterに処理を投げる
                }
            } else {
                warn!(
                    "Couldn't handle acknack/nackfrag! Did not find local RTPS writer with GUID: {:x?}",
                    writer_guid
                );
                continue;
            }
        }
    }
```

### Data
extraflagsってなに？
wiresharkでみると2 octetあって、RustDDSの実装を見ると2 octet幅でどこでも使われてない。~~仕様書を探しても見つからない。~~RTPSのバージョンが上がって、flagが8bitで足りなくなったときのために予約してるやつ。(spec 9.4.5.3.1 p. 159)
octesToInlineQosがあるのも、writerSNとinlineQosの間になにか追加したときのため。
Data Submessageは将来拡張された場合に、後方互換性を持たせるような設計になっている。

RTPS ReaderにRTPS Writerに所属するdata-objectの変更を知らせるSubmessage.

octetsToInlineQosはこのフィールドの直後からinlineQos Elementの最初までのoctet数。もし、inlineQos flagがセットされておらずinlineQosが含まれない場合はこのフィールドの直後からinlineQos Elementの次のElementの最初までのoctet数。

## 用語集
https://fast-dds.docs.eprosima.com/en/latest/fastdds/getting_started/definitions.html
### DDS
DDS domainの中にDomainParticipantとtopicがある。
DomainParticipantの中にPublisher, Subscriberがある。
Publisher, SubscriberはDataWriter/DataReader objectを持つ。
- DCPS entity

    例: Pubulisher, Subscriber

- entity

    例: DataWriter, DataReader, Topic

- DomainParticipant

    Domainに参加している独立したアプリケーション。domain IDによって識別される。

- GUID

    Entityが持つ値

### RTPS
RTPS domainの中にRTPSParticipantがある。
RTPSParticipantの中にwriter, readerがある。
- RTPS Participant

dataを送信, 受信できる要素

- Endpoint

    例: RTPSWriter, RTPSReader

- Topic

    データがどのように交換されるかをラベル付と定義する。
    特定のParticipantに属さない。


## Memo
- socket2 creat

socketに対してunsafeを使わずに詳細な設定をするためのクレート

network/udp_listener.rsでsocket2::Socketを作ってから、UdpSocketを作ってる理由
SO_REUSEADDRを設定するため
https://hana-shin.hatenablog.com/entry/2022/10/18/205924
リスニングソケットではSO_REUSEADRをtrueに設定するのが一般的

- enumflags2 crate
https://github.com/meithecatte/enumflags2

bitflags crateににたAPIを提供するbitflagを扱うためのクレート

