# rs-rtps
RustでRTPS/DDSを実装

最初はStatelessでbesteffortな実装のみに絞って実装する

- [x] DataWriter.write()でData submessageが乗ったパケットを送信できるようにする
- [ ] DataReader.read()でData submessageが乗ったパケットを受信して受け取れるようにする
- [ ] besteffortでほかのノードをDiscoveryしてPub/Subできるところまで実装

## TODO
- [x] QosPoliciesを実装
- [ ] Publisher/Subscriber, DataWriter/DataReader, RTPSWriter/RTPSReaderの役割を把握(DDSがデータを書き込むときに、どこでSubmessageを生成して、どのエンティティーのどのメソッドが呼ばれるのか？)
- [ ] Topicを実装
- [x] Publisher/Subscriberを実装
- [ ] DataWriter/DataReaderのwith_key/no_keyについて調査
- [x] DataWriter/DataReaderを実装
    - [x] RTPSWriterへのコマンドの送信を実装
- [ ] RTPSWriter/RTPSReaderを実装
    - [x] writer_cmd_receiverをevent_loopのpollに登録
    - [x] DataWriterから受け取ったdataのserializerを実装
    - [x] RTPS Messageのビルダーを実装
    - [ ] Best-Effort StatefulWriter Behavior
    - [ ] Reliable StatefulWriter Behavior
    - [ ] Best-Effort StatefulReader Behavior
    - [ ] Reliable StatefulReader Behavior
- [x] UDP senderの実装
- [ ] Discovery Moduleを実装
- [x] HistoryCacheを実装
    - [x] DataWriterにHistoryCacheを実装
    - [x] DataReaderにHistoryCacheを実装
どれもDiscoveryに必要そうなものを最低限実装

## Log
+ パケットの受信を実装

+ 受信したパケットのシリアライズを実装


+ 各モジュールを実装中\
受信したパケットをRTPSプロトコルにしたがって処理するには、RTPS Writer, RTPS Reader, domain participant等のモジュールの実装が必要だから

+ history cache, message builder, message serializerを実装し、datawriter.write(hoge)でhogeをdata submessageにのせて送信可能になった。(HeartBeatは未実装なのでdata submessageだけのmessageしか送れない)
