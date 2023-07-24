# SpecNote
RTPS 2.3 and DDS 1.4のSpecificationを読んだメモ+参考実装を読んだメモ

## 各DDS, RTPSエンティティーの役割
### Publisher/Subscriber (DDS)
DDS spec 2.2.2.4.1 Publisher Class
> A Publisher is the object responsible for the actual dissemination of publications.

RustDDSのsrc/dds/pubsub.rsのDDS Publisherのdocコメント
> The Publisher and Subscriber structures are collections of DataWriters
> and, respectively, DataReaders. They can contain DataWriters or DataReaders
> of different types, and attacehd to different Topics.

DataWriter/DataReaderを生成するためのもの
### DataWriter/DataReader (DDS)
#### DataWriter
DDS spec 2.2.2.4.2 DataWriter Class
> DataWriter allows the application to set the value of the data to be published under a given Topic

data: Dを受け取って、シリアライズしてSerializedPayloadを作成。作成したSerializedPayloadとオプションをチャネル:witer_commandを通じてRTPSWriterに渡す。
送信したい内容をRTPSWriterに渡す

### RTPSWriter/RTPSReader
### RTPSWriter
spec 8.4.2.2 Required RTPS Writer Behaviorを参照
RustDDSのsrc/dds/writer.rsのprocess_writer_commandのコメントの要約
1. DataWriterから受け取ったものをHistoryCacheに追加。
2. データを送信\
    データをpublishしたときはDATA submessageとHEARTBEAT submessageを送信\
    データをpublishしなかったときはHEARTBEAT submessageをだけを送信。このとき、Readerが興味を持っていればDATAとACKNACKを要求してくるはず

## Stateful/Stateless
rtps 2.3 spec 8.4.1 Overview, 8.4.3 Implementing the RTPS Protocol

RTPSの実装はメモリ使用量、帯域使用量、スケーラビリティ、効率の間で異なるトレードオフを選択することとなる。
RTPSの仕様は一致する振る舞いに対して１つの実装を命じない。
代わりにinteroperabilityのために最小の必要条件を定義し、Stateful, Statelessの2つのリファレンス実装を提供する。

### Stateless Reference Implementation:
スケーラビリティに最適化されている。ほとんどリモートエンティティーの状態を保持シないため、巨大システムにおいて良くスケールする。
これは、スケーラビリティを向上させ、メモリ使用量を削減するが、帯域使用量を増加させる。
Statelessな実装はmulticastを通じたbest-effortなコミュニケーションに向いている。
参照: rtps spec 8.4.7.2

### Stateful Reference Implementation:
リモートのエンティティーのすべての状態を保持する。
帯域使用量を最小化するアプローチだが、メモリ使用量か増加し、スケーラビリティが低下する。
Statelessな実装に比べて、厳密で信頼できるコミュニケーションを保証し、Writer側でのQoS-basedもしくはcontent-basedなフィルタリングが可能になる。
参照: rtps spec 8.4.10.2

## Topic
RustDDSでは、src/dds/topic.rsで定義されている。
~~spec探しても情報が見つからん。~~ RTPSじゃなくてDDSのsepcに情報があった。コードのコメントにも"DDS spec 2.3.3"って書いてあるのにRTPSのspecみてた。(https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386)
僕 「Topicって何？」
Chat GPT4 「Topicは、名前（文字列）とデータ型を持ちます。」
RustDDSの実装を確認するとTopicにもたせてるのは名前, DomainParticipant, Qos, Kind(WITH_KEY or NO_KEY)で、データ型は含まれてない。
データ型とTopicを結びつけてるのは、publisher.create_datawriter_cdr::<Shape>(&topic, None)だと思われる。
// TODO: FastDDSがどうやってTopicとデータを結びつけてるのか調べる。
DDSHelloWorldのパケットキャプチャを解析した結果、RTPSを通じてやり取りされるのはTpicの名前のみらしい。具体的なデータ型はソースコードレベルで共有しておいて、
それに紐付いた名前のみをTopicに持たせるっぽい。

(https://fast-dds.docs.eprosima.com/en/latest/fastdds/dds_layer/topic/instances.html)
Topicは1つのデータタイプと紐付けられる。そのため、Topicと関係するデータサンプルはデータ型で示される情報のupdateとして理解される。しかし、論理的に分離して、同じトピック内に、同じデータ型を参照する複数のインスタンスを持つことも可能である。したがって、受信したデータサンプルは、そのTopicの特定のインスタンスに対する更新となる。

## QOS
https://www.omg.org/spec/DDS/1.4/PDF#G5.1034386

## GUID
RTPSEntity トレイトを実装してるやつは持ってる
Participant: 持ってる, RTPSEntity実装
Publisher: EntityIdだけ持ってる
Subscriber: 持ってない
RTPSWriter: 持ってる, RTPSEntity実装
RTPSReader: 持ってる, RTPSEntity実装
DDSWriter: 持ってる, RTPSEntity実装
DDSReader: 持ってる, RTPSEntity実装
### Prefixの決め方
とにかくDomainParticipantのguidから参照

    先頭2 octetはvenderIdの先頭2 octetと同じにする。これによってDDS Domain内で複数のRTPS実装が使われてもguidが衝突しない。残りの 10 octetは衝突しなければどんな方法で生成してもいい。(p. 144)

    RTPS spec 8.2.4.3 The GUIDs of the RTPS Endpoints within a Participant
    > The GUIDs of all the Endpoints within a Participant have the same prefix.

    あるParticipantに含まれるすべてのEndpointのGUIDは同じprefixを持つ。
    > The GUID of any endpoint can be deduced from the GUID of the Participant to which it belongs and its entityId.

    すべてのEndpintのGUIDは所属しているParticipantのGUIDとそのEndpointのentityIdから決定される。

### EntityId, EntityKindの決め方
TODO:

RTPS spec 8.2.4.4 The GUIDs of Endpoint Groups within a Participant
https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A103%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C406%2C0%5D
にPublisher, SubscriberはGUIDを持つと書いてある。
TODO: Publisher, SubscriberにGUIDを実装

RustDDSの実装で、DataWriterとRTPSWriterのGUIDが同じになるようになってるように見える
RTPSのspec 8.2を読んだ感じDDSWriter, DDSReaderはGUIDを持つ必要なさそう。
RustDDSの実装でDDSWriterがRTPSWriterと同じguidを持ってる理由は
1. DDSWtiterがdropするときに一緒に対応するDDSWrtierをドロップするため。
2. write_with_optionsの戻り値(writeから呼び出されるが戻り値は参照されてないので必要ない)
DDSWriterに対してはdropするときに必要になればGUIDを実装すればいいと思う。

RTPSWriterと(DDS)DataWriterはRustDDSの実装を読む限り、１体１対応している。
(TODO: RTPSのspecを確認)

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

## Example Behavior (日本語訳)
specのFigure 8.14 – Example Behavior

https://www.omg.org/spec/DDSI-RTPS/2.3/Beta1/PDF#%5B%7B%22num%22%3A193%2C%22gen%22%3A0%7D%2C%7B%22name%22%3A%22XYZ%22%7D%2C46%2C489%2C0%5D

1. DDSユーザーがDDS DataWriterのwriteオペレーションを呼び出してデータを書き込む。
2. DDS DataWriterが新しいCacheChangeを作るために、RTPS Writerのnew_changeオペレーションを呼び出しす。
3. new_change オペレーションがrutenする。
4. DDS DataWriterがRTPS WriterのHistoryCacheにCacheChangeを保存するためにadd_changeを使う。
5. add_changeオペレーションがrutenする。
6. writeオペレーションがreturnする。
7. RTPS WriterがCacheChangeの変更内容をRTPS ReaderにData Submessageを使って送信し、Heartbeat Submessageを送信してacknowledgemntを要求する。
8. RTPS ReaderがData messageを受信し、リソースの制限が許すと仮定し、add_changeオペレーションを使ってreaderのHistoryCacheにCacheChangeを配置する。
9. add_changeオペレーションがruturnする。CacheChangeはDDS DataReaderとDDSユーザーから見える。この条件はRTPS ReaderのreliabilityLevelアトリビュートに依存する。

    a.  RELIABLE DDS DataReaderには、 RTPS ReaderのHistoryCacheにあるchangeはすべてのそれより前のchange(i.e., より小さいsequence numberをもつchange)が見える場合のみ、ユーザーアプリケーションから見えるようになる。

    b. BEST_EFFORT DDS DataReaderには、RTPS ReaderのHistoryCacheにあるchangeは未来のchangeがまだ見えるようになっていない(i.e., RTPS ReceiverのHistoryCacheにより大きいsequence numberをもつchangeがない)場合のみ、ユーザーから見えるようになる。

〜〜続く〜〜

// TODO