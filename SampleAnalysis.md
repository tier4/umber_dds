# SampleAnalysis
RustDDSを参考実装としてRTPSの解析を行う

## RustDDSのShapdemoからDDSのAPIを確認
Writerの実装を追おうとしたけど、実装を個別に追うのは厳しそうだから、DDSがどんなAPI担っているかを確認して、Writer, DomainParticipant等の関係性を掴む
そのために、examples/shapes_demo/main.rsを読んで見る。
1. domain_idを渡して、DomainParticipantを生成
2. QOSを生成
3. domain_participantの.create_topic()にQOSを渡してtopicを生成。(生成したtopicはmain.rsが持つ)
4. domain_participantからPublisher/Subscriber(以下p/s)を生成。
domain_participantの.create_p/s()にQOSを渡してp/sを生成。(生成したp/sはmain.rsが持つ)
5. p/sからtopicを渡してDaraReader/DataWriter(以下dr/dw)を生成。
その後dr/dwはmainが持つが、p/sはdropされる????
(READER/WRITER)_STATUS_READYをpollに登録
Subscriberの場合readerをpollに登録
6. loop{
    pollに登録したイベントが発生するまで待機
    イベントが発生したら処理して、
    writerを叩いてshape_sampleを送信
}
FastDDSのドキュメント(https://fast-dds.docs.eprosima.com/en/latest/fastdds/getting_started/definitions.html#the-dcps-conceptual-model)の
DDS Domainの図と一致しているが、p/sがdropされるのが理解できない。

## fastDDSのHelloWorldSubscriberからDDSのAPIを確認
class HelloWorldSubscriberを定義
メンバーはDomainParticipant, Subscriber, DataReader, Topic, TypeSuppoert, DataReaderListener.
HelloWorldSubscriberのイニシャライザ
1. QOSを生成
2. QOSを元にparticipantを生成
3. participantにQOSを渡してtopicを生成
4. participantにQOSを渡してsubscriberを生成
5. subscriberにtopic, QOS, DataReaderListenerを渡してreaderを生成

## Topic
RustDDSでは、src/dds/topic.rsで定義されている。
~~spec探しても情報が見つからん。~~ RTPSじゃなくてDDSのsepcに情報があった。コードのコメントにも"DDS spec 2.3.3"って書いてあるのにRTPSのspecみてた。(https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386)

## struct Hoge {inner: Arc<InnerHoge>,} のデザインパターン
複数ヶ所から参照される場合につかう
同一Topicが各DR/DWから参照されるから。
もし、Arcを使わずに、Topicに直接各メンバーをもたせ、
それを各DR/DWから参照しようとすると、ライフタイムを指定してTopic参照を持たせるか、cloneして直接Topicを持たせることになる。

## DDSとRTPSの関係性
FastDDSのドキュメント(https://fast-dds.docs.eprosima.com/en/latest/fastdds/rtps_layer/rtps_layer.html#relation-to-the-dds-layer)
によると、DataReaderとRTPS Readerはわずかな違いだけで一対一対応するとあるが、RustDDSのsrc/dds/reader.rsとsrc/dds/with_key/datareader.rs
の関係性がわからない。

## RTPS Entity
(spec) 8.2.4 The RTPS Entityとsrc/structure/entity.rsが対応

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

DATA, HEARTBEATはspecのFigure 8.14 – Example Behaviorの7番だと思われる。
https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A193%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C489%2C0%5D
DDSCacheのadd_changeにブレークポイントを貼ってみた結果、DATA, HEARTBEATが送信されるのはadd_changeからreturn後だと確認。

~~TODO: このパケットを送信してるコードを見つけ出す~~

-> 多分writer.process_writer_command()とか、ev_wrapper.message_receiver.handle_received_packet(&packet);の中で送信されてる

名前からsend_to_udp_socketでパケットを送信してると思われるから、これにbreakポイント貼って調査

一番最初にsend_to_udp_socketに到達したときのバックトレース
```
#0  rustdds::network::udp_sender::UDPSender::send_to_udp_socket (self=0x7f2020004120, buffer=..., socket=0x7f2020002fe0,
    addr=0x7f2025197880) at src/network/udp_sender.rs:108
#1  0x000055bff628974c in rustdds::network::udp_sender::UDPSender::send_to_locator::{{closure}} (socket_address=...)
    at src/network/udp_sender.rs:129
#2  0x000055bff652b41f in rustdds::network::udp_sender::UDPSender::send_to_locator (self=0x7f2020004120, buffer=...,
    locator=0x7f202000a6e0) at src/network/udp_sender.rs:137
#3  0x000055bff6573f2a in rustdds::dds::writer::Writer::send_message_to_readers (self=0x7f20200088c0,
    preferred_mode=rustdds::dds::writer::DeliveryMode::Multicast, message=0x7f2025198410, readers=...)
    at src/dds/writer.rs:1066
#4  0x000055bff656fe3c in rustdds::dds::writer::Writer::handle_heartbeat_tick (self=0x7f20200088c0, is_manual_assertion=false)
    at src/dds/writer.rs:662
#5  0x000055bff656d501 in rustdds::dds::writer::Writer::handle_timed_event (self=0x7f20200088c0) at src/dds/writer.rs:319
#6  0x000055bff65432f8 in rustdds::dds::dp_event_loop::DPEventLoop::handle_writer_timed_event (self=0x7f2025198b80,
    entity_id=...) at src/dds/dp_event_loop.rs:476
#7  0x000055bff65418a2 in rustdds::dds::dp_event_loop::DPEventLoop::event_loop (self=...) at src/dds/dp_event_loop.rs:349
```
![bt when first reach send_to_udp_socket](https://user-images.githubusercontent.com/58660268/233024270-103cfc1a-ab35-438c-b893-41102d23ada6.png)


## DATA, HEARTBEATのパケットが送信されるまでの流れ
Thread 1: mainが実行されるスレッド
Thread 2: ev_loop_handle, DomainParticipantInnerがこのhandleを持ってる
Thread 3: discovery_handle, DomainParticipantがこのhandleを持ってる

1. DomainParticipant::new(): Thread 1

2. Discovery::new(): Thread 3

    datawriterを生成。このときに、thread 2に対してadd_writer_senderを通じてwriterの生成するように送る
    create_datawriterはThread 3で実行
```
#0  rustdds::dds::pubsub::InnerPublisher::create_datawriter (self=0x7fb93c000ea8, outer=0x7fb949227298, entity_id_opt=...,
    topic=0x7fb9492273d0, optional_qos=...) at src/dds/pubsub.rs:482
#1  0x0000555875018adf in rustdds::dds::pubsub::Publisher::create_datawriter_with_entityid (self=0x7fb949227298, entity_id=...,
    topic=0x7fb9492273d0, qos=...) at src/dds/pubsub.rs:194
#2  0x0000555874c90f8e in rustdds::discovery::discovery::Discovery::new (domain_participant=..., discovery_db=...,
    discovery_started_sender=..., discovery_updated_sender=..., discovery_command_receiver=..., spdp_liveness_receiver=...,
    self_locators=...) at src/discovery/discovery.rs:264
#3  0x0000555874c0f33a in rustdds::dds::participant::DomainParticipant::new::{{closure}} () at src/dds/participant.rs:111
```
3. 2でおくられたシグナルをdp_event_loopで受け取ってhandle_writer_actionがWriter::new()を呼びだす。
```
#0  rustdds::dds::writer::Writer::new (i=..., dds_cache=..., udp_sender=..., timed_event_timer=...) at src/dds/writer.rs:207
#1  0x0000559e1fb30069 in rustdds::dds::dp_event_loop::DPEventLoop::handle_writer_action (self=0x7f8c7269fb80,
    event=0x7f8c7269ff50) at src/dds/dp_event_loop.rs:434
#2  0x0000559e1fb2dde3 in rustdds::dds::dp_event_loop::DPEventLoop::event_loop (self=...) at src/dds/dp_event_loop.rs:259
#3  0x0000559e1f772703 in rustdds::dds::participant::DomainParticipantInner::new::{{closure}} () at src/dds/participant.rs:767
```
4. dp_event_loopでpollがTokenDecode::Entity(eid)のイベントを受け取ったとり、process_writer_commandが呼ばれる。
// TODO: このイベントがどこ由来か調査

5. process_writer_commandからDDSCache::add_change()が呼ばれる。

    add_changeはThread 2で実行
    一番最初にadd_changeに到達したときのバックトレース
    ![bt when first reach add_change](https://user-images.githubusercontent.com/58660268/233294379-4d8c40c1-6db7-4413-aba1-db92d57017ea.png)

6. add_change()からreturnした後にrustdds::dds::writer::Writer::process_writer_commandまでもどってsend_to_udp_socketが呼ばれる。

    send_to_udp_socketはThread 2で実行


RustDDSのsrc/dds/reader.rs, writer.rsはそれぞれRTPS Reader, RTPS Writerの実装。


## 解析の感想
20スレッドが非同期で動いて、各オブジェクトの状態が把握しづらいから辛い。
どのタイミングで何が呼ばれているのかが把握し辛い。
関数の呼び出しを追っていって深くなってくると、自分の頭の中のスタックがオーバーフローしてる感じがする。
非同期で動いているからデバッガでブレークするとパケット送信の順番が変わるっぽい。

## 最初の4つのINFO_TS, HEARTBEATが送られる理由
RTPS spec 8.4.2.2 Required RTPS Writer Behavior

8.4.2.2.3 Writers must send periodic HEARTBEAT Messages (reliable only) 
(https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A198%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C172%2C0%5D)
これだと思ったけど、each matching reliable Readerに対して HEARTBEATを送るとあるから、multicastしてるのはおかしくない？
handle_heartbeat_tick()のコメントにこれは周期的に呼ばれるって書いてある。
コードからreliable only要素が読み取れない。

厳密な信頼性のある通信のために、Writerは、Readerが利用可能なすべてのサンプルの受信をacknowledgeするか、またはReaderが消失するまで、Readerに対してHEARTBEATメッセージを送り続けなければならない。それ以外のケースでは、送信されるHEARTBEATメッセージの数は実装固有であり、有限である。

~~TODO: 以下を調査~~
どうしてINFO_TSがセットなのか？
-> spec読んだけど見つからない
specにはreliable onlyの場合とあるのに、コードからreliable only要素が読み取れないのはなぜか？
-> WriterのQOSが常にreliableになるのかと思ったけど、Writer::new()のheartbeat_periodをprintしてみたら、Noneになったから、違う。
どうして、multicastで送られるのか？
-> FastDDSを使ったShapeDemo一番最初に送られるのはINFO_TS, DATA, Unknowで、INFO_TS, HEARTBEATは送られてない。RustDDSの実装が仕様に従ってないだけの可能性が高い。

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
6. 既知だが、無効なSubmessageは残りのMessage(the rest of the Message)を無効にする。
// "the rest of the Message"が何を指すのか仕様書から読み取れないが、RustDDSの実装は、無効なSubmessageが含まれるMessageを無効なものとして破棄している
tomiy(tomiy-tomiylab)とytakano(ytakano)の解釈はそれまでに処理したSubmessageは使用し、無効なSubmessageとそれより後のSubmessageを破棄する。
ただし、無効なSubmessageを受け取ると、それ以降のSubmessageを無効とするとしか仕様書には書いておらず、Submessageを無効とする具体的な操作は定義されていない。
同一Message内に複数のSubmessageが含まれている場合、前のSubmessageは後ろのSubmessageを処理するのに必要な情報である。
"8.3.4 The RTPS Message Receiver
The interpretation and meaning of a Submessage within a Message may depend on the previous Submessages contained
within that same Message. "
つまり、Message内に1つでも無効なSubmessageが含まれている場合、そのMessageを処理する意義は失われるため、RustDDSでは破棄していると思われる。

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
このSubmessageはReaderが受信したシーケンス番号とまだ喪失しているシーケンス番号をWriterに伝えることを可能にする。
このSubmessageはACKとNACKの両方に使える。

AckNackは２つの目的を同時に提供する。
- The Submessage acknowledges all sequence numbers up to and including the one just before the lowest sequence number in the SequenceNumberSet (that is readerSNState.base -1).
- AckNackは、SequenceNumberSet内の最も小さいシーケンス番号（つまりreaderSNState.base -1）の直前までのすべてのシーケンス番号を伝える。
- The Submessage negatively-acknowledges (requests) the sequence numbers that appear explicitly in the set.
- AckNackは、セットに明示的に含まれるシーケンス番号を要求(negatively-acknowledges)する。
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

## Writer
RustDDSではsrc/dds/writer.rsで定義されている。
コード読んでみても何もわからない。
spec 8.4.2.2 Required RTPS Writer BehaviorにWriterの挙動について書いてある。

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

