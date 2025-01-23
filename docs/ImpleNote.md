# ImpleNote
note about UmberDDS implementation

## Module description
| module | description |
----|----
| dds | DDS API |
| rtps | RTPS implementation |
| discovery | RTPS Discovery Module |
| message | RTPS Message Module |
| strcture | structures for DDS & RPS |
| netwok | network utils |

## Module Oraganization
![UmberDDS_modules](https://github.com/user-attachments/assets/76621ab4-bf78-42a8-8792-bc08c0dfdde5)


## architecture description
UmberDDS is designed to operate in an event-driven manner by using an EventLoop that waits for events such as network and mpsc channel events. This EventLoop is executed in a thread spawned when constructing `dds::Participant` during the DDS app's startup to join the Domain. Simultaneously, a Discovery thread is also spawned, where the Discovery Module is executed.

All packets directed to UmberDDS are received by the EventLoop, serialized by the Message module, and then distributed to the respective entities for processing.

The results of discovery are communicated from the Discovery Module to the EventLoop through the DiscoveryDB. The DiscoveryDB is wrapped in an Arc and shared between the EventLoop and the Discovery Module. When the Discovery Module updates the DiscoveryDB, it notifies the EventLoop via an mpsc channel. Upon receiving this notification, the EventLoop configures the settings for each entity.

## dds module
In the DDS specification, each entity is defined as a class that inherits from an abstract class. However, inheritance does not exist in Rust. Therefore, in this implementation, all methods and properties of each superclass are implemented directly in the structs representing each entity. As a result, unused properties may exist in some structs.

### datareader
Provide topic subscribing interface to DDS app.

### datawriter
Provide topic publishing interface to DDS app.

### event_loop
The central event-driven loop. Network sockets, timers, and mpsc channels are registered with mio's Poll, and this loop monitors the Poll.

### participant
A factory of Publisher adn Subscriber.

GuidPrefix is the identifier for a DomainParticipant and must be unique within the domain. Ensuring absolute uniqueness can be costly, so in this implementation, absolute uniqueness is not guaranteed. Instead, a random number is used as the GuidPrefix to keep the probability of collisions sufficiently low. Additionally, since this implementation aims to run in a no_std environment, small_rng is used for random number generation. Therefore, a small_rng is required as an argument when creating a DomainParticipant.

DomainParticipant is referenced and modified by multiple structures, such as Publisher and Discovery, so it needs to be wrapped in an Arc<Mutex<>>. Additionally, to terminate the Discovery thread when the DDS app ends, DomainParticipantDisc is inserted between DomainParticipant and DomainParticipantInner. However, the current implementation does not include the process for terminating the Discovery thread.

TODO: Terminate the Discovery thread when the DDS app ends.


### publisher
A factory of DataWriter.

### subscriber
A factory of DataReader.

## rtps module

### writer
Implementation of RTPS stateful Refarence Writer Behavior.

### reader
Implementation of RTPS stateful Refarence Reader Behavior.

### cache

## network module
related UDP socket

## discovery module
The Discovery module is responsible for publishing and subscribing to built-in topics to notify remote Entities of its own existence and to discover the existence of remote Entities.

As stated in the RTPS specification, communication is performed in the same manner as in a regular application: a participant creates publishers and subscribers, from which datawriters and datareaders are created, and information is exchanged using those datawriters and datareaders.

In this implementation, the Simple Discovery Protocol (SDP) as defined by the RTPS standard is implemented. SDP consists of two stages: the Simple Participant Discovery Protocol (SPDP) for participant discovery and the Simple Endpoint Discovery Protocol (SEDP) for entity discovery.

SPDP multicasts SPDP messages to inform other Participants of a Participant's existence and learns about the existence of other Participants by receiving SPDP messages. It also monitors the liveness of other Participants (this feature is not implemented in UmberDDS yet).

In this implementation, received SPDP data is registered in the discovery database (discovery_db), and updates are notified to the event loop through a channel. Upon receiving this, the event loop configures the sedp_builtin entity.

SEDP communicates the Entities it holds to remote Participants discovered by SPDP. By learning about the existence of remote Entities through SEDP and configuring its own Entities, mutual communication becomes possible.

In this implementation, upon receiving SEDP, the entity is configured within the message_receiver.
