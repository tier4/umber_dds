# rs-rtps
RustでRTPS/DDSを実装

- [ ] DataWriter.write()でData submessageが乗ったパケットを送信できるようにする
- [ ] DataReader.read()でData submessageが乗ったパケットを受信して受け取れるようにする
- [ ] besteffortでほかのノードをDiscoveryしてPub/Subできるところまで実装

## TODO
- [x] QosPoliciesを実装
- [ ] Publisher/Subscriber, DataWriter/DataReader, RTPSWriter/RTPSReaderの役割を把握(DDSがデータを書き込むときに、どこでSubmessageを生成して、どのエンティティーのどのメソッドが呼ばれるのか？)
- [ ] Topicを実装
- [ ] Publisher/Subscriberを実装
- [ ] DataWriter/DataReaderのwith_key/no_keyについて調査
- [ ] DataWriter/DataReaderを実装
    - [ ] RTPSWriterへのコマンドの送信を実装
- [ ] RTPSWriter/RTPSReaderを実装
    - [ ] writer_cmd_receiverをevent_loopのpollに登録
- [ ] UDP senderの実装
- [ ] Discovery Moduleを実装
- [ ] HistoryCacheを実装
    - [ ] DataWriterにHistoryCacheを実装
    - [ ] DataReaderにHistoryCacheを実装
どれもDiscoveryに必要そうなものを最低限実装

## Log
+ パケットの受信を実装

+ 受信したパケットのシリアライズを実装


+ 各モジュールを実装中\
受信したパケットをRTPSプロトコルにしたがって処理するには、RTPS Writer, RTPS Reader, domain participant等のモジュールの実装が必要だから
