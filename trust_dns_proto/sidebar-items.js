initSidebarItems({"mod":[["error",""],["op","Operations to send with a `Client` or server, e.g. `Query`, `Message`, or `UpdateMessage` can be used to gether to either query or update resource records sets."],["rr","Resource record related components, e.g. `Name` aka label, `Record`, `RData`, ..."],["serialize","Contains serialization libraries for `binary` and text, `txt`."],["tcp","TCP protocol related components for DNS"],["udp","UDP protocol related components for DNS"]],"struct":[["BasicDnsHandle","Root DnsHandle implementaton returned by DnsFuture"],["BufDnsStreamHandle","A buffering stream bound to a `SocketAddr`"],["BufStreamHandle","A sender to which serialized DNS Messages can be sent"],["DnsFuture","A DNS Client implemented over futures-rs."],["RetryDnsHandle","Can be used to reattempt a queries if they fail"],["StreamHandle","The StreamHandle is the general interface for communicating with the DnsFuture"]],"trait":[["DnsHandle","A trait for implementing high level functions of DNS."],["DnsStreamHandle","Implementations of Sinks for sending DNS messages"]],"type":[["MessageStreamHandle","A sender to which a Message can be sent"]]});