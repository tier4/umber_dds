# SampleAnalysis
RustDDSを参考実装としてRTPSの解析を行う
たまにFastDDSも読む

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

## mio::poll
### v0.8.6 (本実装で使用)
poll.registry().register()
第一引数の監視対象: source: &mut S (S: Source + ?Sized)

### v0.6.23 (RustDDSで使用)
poll.register()
第一引数の監視対象: handle: &E (E: Evented + ?Sized)

## mio_extra ( https://github.com/dimbleby/mio-extras )
mioとは別の開発者が開発
mioのv0.6にしか対応していない

## mio_channel ( https://github.com/oh-jinsu/mio-channel )
これもmioとは別の開発者が開発
mioのv0.8に対応しているが、SyncSenderに非対応
自分で対応させてPR送った

## mioでchannelを使うには
- mio_channelをSyncSenderに対応させるpull reqを送って、mio_channelを使う
- ~~mio_channelをforkしてSyncSenderに対応させたクレートを公開して使用~~
- ~~自前でmio_chennelのようなmpscのmio wrapperを実装~~
- ~~mio本体にmpscのwrapperを追加するpull reqを送る~~

## RustDDSのchennel
+ add_writer:\
    sender: Publisher\
    receiver: DPEventLoop
+ witer_command:\
    receiver: RTPS Writer (DpeventLoopでpoll)\
    sender(cc_upload): DataWriter

## fastDDSのHelloWorldSubscriberからDDSのAPIを確認
class HelloWorldSubscriberを定義
メンバーはDomainParticipant, Subscriber, DataReader, Topic, TypeSuppoert, DataReaderListener.
HelloWorldSubscriberのイニシャライザ
1. QOSを生成
2. QOSを元にparticipantを生成
3. participantにQOSを渡してtopicを生成
4. participantにQOSを渡してsubscriberを生成
5. subscriberにtopic, QOS, DataReaderListenerを渡してreaderを生成

# RustDDSのスレッド
## Tread 1 (main)
DomainParticipantを所有
pollに{DataWriter, DataReader}を登録してloopでイベントを処理DomainParticipantはadd_writer_senderを所有(Publisherに配布するため)
DomainParticipantのcreate_{publisher/subscriver}で{publisher/subscriver}を作成
{Publisher/Subscriber}のcreate_data{writer/reader}でdata{writer/reader}を作成。
このとき、{reader/writer}_command_{sender/reveiver}を作成し、senderはdata{writer/reader}にもたせる。
create_data{writer/reader}で{writer/reader}の作成命令を{Publisher/}の持つadd_{writer/reader}_senderに送る。ことのきに{reader/writer}_command_{sender/reveiver}も送る。


## Tread 2 (DPEventLoop)
handler: thread 1のDomainParticipantが所有

pollにいろいろ登録してloopでイベント処理
pollに登録されているもの
- 全UDPソケット
- {add/remove}_{reader/writer}_receiver
- stop_poll_receiver
- acknack_receiver
- discovery_update_notification_receiver
add_writer_receiverでThread1のcreate_datawriterから受信

![IMG_3698](https://github.com/tier4/T4RustDDS/assets/58660268/03b4b425-8156-4de5-9ce4-07e7b8dfbb5c)

## 独り言
エンティティーを生成するときに、データの依存関係があって
あるデータを持ってるのは〜で、あるデータを生成するのが〜で
ってなってて正しい順番、正しいデータの流れじゃないとエンティティーの生成に必要な情報が揃わない。
データの依存関係とデータの流れがスパゲッティみたいに絡み合ってて解析するの辛い。
GUIDとか、がなんのためのものかとかの各プロパティの役割を理解すると解析が楽になりそう。
もうちょっとマクロな視点でDDS, RTPSを理解するといいかもって思った。
けど、specから読み解くのは難しいし、RustDDSの実装から理解するのが難しくて詰まってるんだから実装から読み解くのは難しい

## メモ
RustDDSのsrc/dds/pubsub.rsを読んでると
publisherのcreate_data_writerはDataWriter作ると同時にRTPSWriterを作るように
participantに送信してる。
このときにDataWriterとRTPSWriterをチャネルでつなげてる。

## Channel
mio_extraにはsync_channelとchannelがある。
それぞれの違いをメモ
mio_extraには有益な情報がなかったが、
mio_extraのchannelはstd::sync::mpsc::channelをwrapしてるだけ
だからそっちのドキュメントを参照。
### chnnel
Senderから送信されたすべてのデータは、送信された順番でReceiverで利用可能になる。sendを呼び出したthreadはブルックされない。(このチャネルは無限大のバッファを持っている。)
Senderが1つでも生きている間、messageが受信可能になるまでrecvはブロックされる。
### sync_channel
SyncSenderから送信されたすべてのデータは、送信された順番でReceiverで利用可能になる。channelと同じでReceiverはメッセージが利用可能になるまでブロックされる。
バッファのリミットに到達するとsendの呼び出しをブロックする。
sync_channelはsenderのsemanticsが大きく異なる。
buffer sizeを0にするのは許容され、その場合対になっているrecvが実行されるまでsendはブロックされる。

## Publisher
RustDDSのPublisherがもってるdefault_dw_qosについて、(https://fast-dds.docs.eprosima.com/en/latest/fastdds/dds_layer/publisher/publisher/publisher.html#default-publisherqos)を参照
DDSのspecの(https://www.omg.org/spec/DDS/1.4/PDF#G5.1030755)に詳細

CDRはCommaon Data Representationの略だと思われる
http://www.omg.org/cgi-bin/doc?formal/02-06-51
にCDRについて書いてある。

## with_key/no_keyについて
keyがなんのkeyなのかはわからなかったが、no_keyはwith_keyをwrapしてるので
with_keyを実装して必要になればno_keyを実装する。
fastddsgenで生成したコード読んでたら`bool HelloWorld::isKeyDefined()`
というメソッドがあったからKeyの概念自体はDDS共通のものだと思われる。
DDS specの2.2.2.4.2.9 get_key_valuにinstance keyというフレーズがある。
https://fast-dds.docs.eprosima.com/en/latest/fastdds/dds_layer/topic/instances.html
と照らし合わせると同じトピック内に、同じデータ型を参照する複数のインスタンスを識別するためのものだと思われる。Instanceってなんのこと？
TopicがClassだとするとそれから生成されるObjectということになる。
すると、TopicをPublisherとかに持たせるときは参照にしてむやみにInstanceを生やさないのが正しいかもしれない。datawriterを新たに生成してtopicを保持するときはInstanceを持っていいけど、topicを取得するときは参照を返す。
dispoedはRustのDropに対応するものだと思う。RustDDSもDataWriterにDropトレイトを実装してた。

## DataReader/DataWriter
RustDDSのREADME.mdより
### Data serialization and keying
いくつかの既存のDDS実装はそれぞれのペイロードタイプのDataReader/DataWriterの実装のためにコード生成を使用している。
> DataReader/DataWriterはそれぞれのデータタイプごとに実装されるものらしい

私達はコード生成に頼らない。そのかわり、Rust generic programmingを使う。
Serdeライブラリがペイロードのデータのシリアライズ/デシリアライズに使われる。

## Writer
dp_event_loopが所有
entity_idをtokenに変換して、writer_cmd_receiverをpollに登録するときのTokenにする。

## Reader
message_receiverが所有
DataWriterが持っているchacheにどうやって受け取ったデータを書き込んでるかは要調査(TODO)

## Data submessage
これまでユーザーがpublishしたデータを載せるためのものだと思ってたけど、そうじゃないみたい。
パケットキャプチャを見てたらユーザーがpublishしたデータ以外がserializedpayloadにあった。
Wireshark上でData(p)にはprotocol_versionとかVenderIDとかが含まれてて、Data(w)にはTopic名とかunicast_locatorとかが含まれてるから、
ディスカバリーのためにプロトコルの情報とか、Writerの情報を送ってるっぽい。
RTPS spec 2.4には"Contains information regarding the value of an application Date-object."
「アプリケーションデータオブジェクトの値に関する情報を含む」としか書いてない。

## HistoryCache
structure/cache_change.rsに実装がある。
writer chaceはDataWriterがreader cacheはDataReaderが持っている。

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
$ lsof -i -n -p
shapes_de 160 root    3u  IPv4 450292      0t0  UDP *:7400
shapes_de 160 root    4u  IPv4 450294      0t0  UDP *:7410
shapes_de 160 root    5u  IPv4 450295      0t0  UDP *:7401
shapes_de 160 root    6u  IPv4 450297      0t0  UDP *:7411
shapes_de 160 root   10u  IPv4 457163      0t0  UDP *:35442
shapes_de 160 root   11u  IPv4 457165      0t0  UDP 192.168.208.3:53923 // このポートは毎回変わる
                                                                        // このポート番号を以下rpとする
```
各ポートの意味(domainId = 0, participantId = 0)
- 7400: spdp_multicast_port
- 7410: spdp_unicast_port
- 7401: usertraffic_multicast_port
- 7411: usertraffic_unicast_port

開かれるportはp. 165にある。RustDDSのポートを決める実装はsrc/network/constant.rsにあり、仕様書のdefault通りに実装されている。
上4つは受信用, 下2つは送信用。
下２つの送信用のうち、上側はmulticast,下側はunicast
受信用はsubscribeのため、送信用はdiscoveryのため。

RustDDSの実装ではUDPSenderのSocketに対してset_multicast_loop_v4(true)
がセットされてる。コメントに
> We set multicasting loop on so that we can hear other DomainParticipant
> instances running on the same host.

と書いてあるが、どこにもsenderで受信してる処理はない。
-> "other DomainParticipant instances running on the same host"がこのSenderが送信したpacketを受信できるようにするため。 なぜここで"we"が主語なのかは不明。

## UDP socketにセットするオプション
### UDP Listener
+ reuse_address: true

同一ホスト上のほかのインスタンスが同一のmultcast addressとportを使用できるようにするため。
socketにaddressをbindする前にセット
+ nonblocking: true

### UDP Sender
+ multicast_if_v4 (multi cast only)
+ multicast_loop_v4: true
同一ホスト上のほかのインスタンスがこのSenderが送信したpacketを受信できるようにするため。

## Socket APIでUDP Multicastを送受信する手順
### sender
+ UDP socketを生成
+ multicast_if_v4でmulticastの送信先インターフェースのアドレスを指定
+ 送信元のaddressとportをsocketにbind
+ 同一ホスト上のほかのインスタンスがmulticastのdatagramを受信する必要がある場合、multicast_loop_v4にtrueをセット
+ socket.send_to(data, "{multicast_address}:{dest_port}")でmulticast_addressのmulticast groupにペケットを送信
### receiver
+ UDP socketを生成
+ 同一ホストで同一のmultcast addressとportを使用する場合、reuse_addressにtrueをセット
+ addressとportをbind(addressは"0.0.0.0"で良い)
+ 非同期で受け取る場合、nonblockingにtrueをセット
+ join_multicast_v4({multicast_group}, {interface_addres})でmulticastグループに加入させ、そのインターフェースでmulticastのパケットを受信できるようにする。
+ recev_from(buf)で受信

## UDP multicastの受信に関して
同一ホストの別インスタンスがUDP multicastのdatagramを受信できるように、multicast_loop_v4にtrueを設定しているため、
自分自身が送信したdatagramを受信してしまうが、DUPソケットがdatagramに対して同一インスタンスから送信されたものかを判別できないため、これは仕様。
RTPSのレイヤーでGUID_Prefixを確認すれば送信元のインスタンスを区別できるため、そこでうまく処理する。

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
    let mut participant_id = 0;
    let mut discovery_listener = None;
    while discovery_listener.is_none() && participant_id < 120 {
        discovery_listener = UDPListener::new_unicast(
            "0.0.0.0",
            spdp_well_known_unicast_port(domain_id, participant_id),
        ).ok();
        if discovery_listener.is_none() {
            participant_id += 1;
        }
    }
    // "0.0.0.0:spdp_well_known_unicast_port(domain_id, , participant_id)"を開く
    // participantIdはポートの衝突を避けるため、同一domain、同一nodeで一意である必要がある。
    // socketが開けるか確認して、開けなければparticipantIdを1増やす

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
ポート7400はdomainId=0のときのデフォルトのspdp_multicast_port
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
Example Behaviorの1はユーザーがデータを送信で始まってるけど、このケースではメッセージが送信されるトリガーはユーザーの書き込みではない。
Discoveryがdata_writeを生成し、それに付随して送信されたものと思われるが、流れを追うのが難しく確信が持てない。

Thread 1: mainが実行されるスレッド
Thread 2: ev_loop_handle, DomainParticipantInnerがこのhandleを持ってる
Thread 3: discovery_handle, DomainParticipantがこのhandleを持ってる

1. DomainParticipant::new(): Thread 1

2. Discovery::new(): Thread 3

    datawriterを生成。このときに、thread 2のdp_event_loopが持っているadd_writer_receiverに対してadd_writer_senderを通じてwriterの生成するように送る
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
4. handle_writer_actionでWriterを生成し、pollにいろいろ登録
5. dp_event_loopでpollがTokenDecode::Entity(eid)のイベントを受け取り、process_writer_commandが呼ばれる。\
このイベントの発生源は4. でpollに登録したやつっぽい。\
TODO: なんのためのそれらのイベントがpollに登録されたのか調査 -> Discoveryのための可能性大, specのDiscoveryを読む
TokenDecode::Entity(eid)のイベントを受け取ったreceiverはdp_event_loop.rs:386のやつで、これと対になるsenderは2.
のcreate_datawriterが生成したDataWriterが持ってる。\
こいつに書き込まれるのはsrc/discovery.rsのdiscovery_event_loop()の中のwrite_writers_info()。
self.dcps_publication_writer.write() (selfはDiscovery)だから https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A305%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C553%2C0%5D の"DCPSPublication"が関係してる。
詳しくはDDSのspecを確認。
```
There are four pre-defined built-in Topics: “DCPSParticipant,” “DCPSSubscription,” “DCPSPublication,” and
“DCPSTopic.” The DataTypes associated with these Topics are also specified by the DDS specification and mainly
contain Entity QoS values.
```
6. process_writer_commandからDDSCache::add_change()が呼ばれる。

    add_changeはThread 2で実行
    一番最初にadd_changeに到達したときのバックトレース
    ![bt when first reach add_change](https://user-images.githubusercontent.com/58660268/233294379-4d8c40c1-6db7-4413-aba1-db92d57017ea.png)

7. add_change()からreturnした後にrustdds::dds::writer::Writer::process_writer_commandまでもどってsend_to_udp_socketが呼ばれる。

    send_to_udp_socketはThread 2で実行


RustDDSのsrc/dds/reader.rs, writer.rsはそれぞれRTPS Reader, RTPS Writerの実装。

この最初に送られるData SubmessageのserializedDataを見たら https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A309%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C489%2C0%5D と概ね一致していた。

## domain_participantの生成からdatawriter, rtpswriter生成までの流れ
### domain_participantを生成
domain_participantのメンバー
```
add_writer_sender
my_guid
```
add_writer_*を生成
add_writer_receiverはdp_event_loopに渡す
domain_participantがadd_writer_senderを保持するのはpublisherに配布するため

### domain_participantからpublisherを生成
add_writer_senderとparticipant自身をPublisher::new()に渡す
Publisherはdomain_participantとadd_writer_senderを保持

### publisherからdatawriterを生成と同時にrtpswriter生成
publisherのcreate_datawriterでwriter_command送信用のチャネルを生成。
このチャネルはdata_writerがwriterにデータを送るためのもの
自身の保持してるadd_data_writerにwriterを生成するように送信
このときwriter_command送信用のチャネルのreceiverをwriterに渡す。
data_writerを生成
このときwriter_command送信用のチャネルのsenderをwriterに渡す。

## The Simple Participant Discovery Protocol
spec 8.5.3 \
プログラムを起動して一番最初に送られるパケットはこのDiscovery Endpointのためのものだから、このプロトコルについて調査する。

Chat GPT-4によるThe Simple Participant Discovery Protocoの要約(間違いは修正)


>     1. パーティシパントの生成: パーティシパントがドメインに参加すると、それぞれのParticipantを区別するためのParticipantIdが割り当てられます。
>
>     2. マルチキャスト送信: 新しいパーティシパントは、その存在を周知させるために、所定のマルチキャストアドレスにParticipantDataメッセージを送信します。このメッセージには、プロトコルバージョン、ベンダーID、リスナーアドレス、およびその他のメタデータが含まれます。
>
>     3. メッセージ受信: 既存のパーティシパントは、マルチキャストアドレスからParticipantDataメッセージを受信し、新しいパーティシパントの情報をディスカバリデータベースに追加します。
>
>     4. 応答: 既存のパーティシパントは、新しいパーティシパントに対して独自のParticipantDataメッセージを送信し、自身の情報を通知します。これにより、新しいパーティシパントも既存のパーティシパントの情報をディスカバリデータベースに追加できます。
>
>     5. 定期的なアナウンス: パーティシパントは定期的に、自身の存在とデータをマルチキャストアドレスに送信します。これにより、新たに参加したパーティシパントや情報が失われたパーティシパントが、他のパーティシパントの情報をディスカバリデータベースに追加できます。
>
> このような流れで、SPDPはドメイン内のパーティシパント同士のディスカバリを行い、RTPSにおいてリアルタイム通信の基盤を提供します。

僕 「2.の手順についてマルチキャストアドレスにParticipantDataを送信するのはどのWriterですか？」\
Chat GPT 「マルチキャストアドレスにParticipantDataメッセージを送信するのは、特別なビルトインデータライターであるSPDPdiscoveredParticipantDataWriterです。このビルトインデータライターは、各パーティシパントによって自動的に生成され、パーティシパントのディスカバリプロセスをサポートします。

SPDPdiscoveredParticipantDataWriterは、新しいパーティシパントがドメインに参加したときや定期的なアナウンスの際に、ParticipantDataメッセージを所定のマルチキャストアドレスに送信します。これにより、他のパーティシパントが新しいパーティシパントの存在を検出し、相互に通信できるようになります。」

## ディスカバリーのやり取りをパケットキャプチャしたメモ
P2: subscribe, 2, P1:publish, 3
2つのParticipant, P1, P2のPDPについてパケットキャプチャで観察してみた。
P1がブロードキャストでPDPメッセージを送信, BUILTIN_SDP_PARTISIPANT_WRITER
P2がブロードキャストでPDPメッセージを送信, BUILTIN_SDP_PARTISIPANT_WRITER
P2がP1からのブロードキャストPDPメッセージを受信, BUILTIN_SDP_PARTISIPANT_READER
P2がP1にユニキャストPDPメッセージを送信
P1がP2からのブロードキャストPDPメッセージを受信
P1がP2からのユニキャストPDPメッセージを受信
ここでPDPは完了？
ここまではINFO_TS, DATA(p)でやりとりされる。
ここから、
P2がP1にINFO_DST, HERARBEATを送信、BUILTIN_PUBLICATIONS_WRITER
P1がP2からINFO_DST, HERARBEATを受信、BUILTIN_PUBLICATIONS_READER
P1がP2にINFO_DST, ACKNACKを送信, BUILTIN_PUBLICATIONS_WRITER
これを、BUILTIN_SUBSCRIPTIONS_WRITER, P2P_BUILTIN_PARTICIPANT_MESSAGE_WRITERにかんしてやり取りしてる
HEARTBEATの中身に特に意味のあるデータはなさそう
その後、P2からP1にユニキャストでINFO_DST, INFO_TS, DATA(r)が送信されてる, BUILTIN_SUBSCRIPTIONS_WRITER
このDATA(r)の中身を見た感じ、Topic nameとかが入ってるからSEDPのパケットっぽい
RTPS 2.3 specの8.5.4.2 The built-in Endpoints used by the Simple Endpoint Discovery Protocol図をみると、BUILTIN_PUBLICATIONS_WRITER, BUILTIN_SUBSCRIPTIONS_WRITERはSEDPのためのエンドポイントで間違いなさそう。

## SPDPとSEDP
SPDPの情報はDiscovery Moduleのdiscovery_dbに、SEDPの情報は各Writerのreader_proxyで管理される
(StatefullなwriterはSEDPの情報を各Writerのreader_proxyで管理するが、StatelessなWriterでは、ReaderLocatorで管理される)
rtps spec 8.4 Befavior Moduleを参照

RustDDSではStatefullな実装を採用しているらしい

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
// Discovery threadはDomainParticipantで生成される。そのhandlerは
// DomainParticipantDiscのdiscovery_join_handle: mio_channel::Receiver<JoinHandle<()>>に対して送りつけられる。
// DomainParticipantDiscがchannelからhandlerを受信するのはDomainParticipantDiscがdropされたとき。
// DomainParticipantDiscがdropされると、handlerを受信してjoinし、スレッドを終了させる。
// Discovery threadはDomainParticipantの参照を持つ必要があり、そのスレッドはDomainParticipantがdropされるときに一緒にdropされる必要があるため
// DomainParticipantが持つ必要がある。この関係を満足するために、スレッドのhandlerをmio_channelに対して送信し、
// 受信せずにバッファに保持させておくというやり方に驚いたのでメモを残しておく。

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

MessageReceiver::new()で*_reply_locator_listの初期値が`vec![Locator::Invalid]`になっている。しかし、仕様書のp. 38にはLocatorの初期値には受信したメッセージにしたがって値をセットすると書いてあるから、RustDDSが初期値をInvalidに設定している理由を調査。

-> "The list is initialized to contain a single Locator_t with the LocatorKind,"と書いてあるから要素を1つ含むVecとして初期化しないといけない。
しかし、コンストラクターを実行するのは受信前だからアドレスもポートも設定できないからINVALID一つを要素として初期化している。


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

- Endpoint (https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A107%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C383%2C0%5D)

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

