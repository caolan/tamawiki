initSidebarItems({"constant":[["ACCEPT","Advertises which content types the client is able to understand."],["ACCEPT_CHARSET","Advertises which character set the client is able to understand."],["ACCEPT_ENCODING","Advertises which content encoding the client is able to understand."],["ACCEPT_LANGUAGE","Advertises which languages the client is able to understand."],["ACCEPT_RANGES","Marker used by the server to advertise partial request support."],["ACCESS_CONTROL_ALLOW_CREDENTIALS","Preflight response indicating if the response to the request can be exposed to the page."],["ACCESS_CONTROL_ALLOW_HEADERS","Preflight response indicating permitted HTTP headers."],["ACCESS_CONTROL_ALLOW_METHODS","Preflight header response indicating permitted access methods."],["ACCESS_CONTROL_ALLOW_ORIGIN","Indicates whether the response can be shared with resources with the given origin."],["ACCESS_CONTROL_EXPOSE_HEADERS","Indicates which headers can be exposed as part of the response by listing their names."],["ACCESS_CONTROL_MAX_AGE","Indicates how long the results of a preflight request can be cached."],["ACCESS_CONTROL_REQUEST_HEADERS","Informs the server which HTTP headers will be used when an actual request is made."],["ACCESS_CONTROL_REQUEST_METHOD","Informs the server know which HTTP method will be used when the actual request is made."],["AGE","Indicates the time in seconds the object has been in a proxy cache."],["ALLOW","Lists the set of methods support by a resource."],["ALT_SVC","Advertises the availability of alternate services to clients."],["AUTHORIZATION","Contains the credentials to authenticate a user agent with a server."],["CACHE_CONTROL","Specifies directives for caching mechanisms in both requests and responses."],["CONNECTION","Controls whether or not the network connection stays open after the current transaction finishes."],["CONTENT_DISPOSITION","Indicates if the content is expected to be displayed inline."],["CONTENT_ENCODING","Used to compress the media-type."],["CONTENT_LANGUAGE","Used to describe the languages intended for the audience."],["CONTENT_LENGTH","Indicates the size fo the entity-body."],["CONTENT_LOCATION","Indicates an alternate location for the returned data."],["CONTENT_RANGE","Indicates where in a full body message a partial message belongs."],["CONTENT_SECURITY_POLICY","Allows controlling resources the user agent is allowed to load for a given page."],["CONTENT_SECURITY_POLICY_REPORT_ONLY","Allows experimenting with policies by monitoring their effects."],["CONTENT_TYPE","Used to indicate the media type of the resource."],["COOKIE","Contains stored HTTP cookies previously sent by the server with the Set-Cookie header."],["DATE","Contains the date and time at which the message was originated."],["DNT","Indicates the client's tracking preference."],["ETAG","Identifier for a specific version of a resource."],["EXPECT","Indicates expectations that need to be fulfilled by the server in order to properly handle the request."],["EXPIRES","Contains the date/time after which the response is considered stale."],["FORWARDED","Contains information from the client-facing side of proxy servers that is altered or lost when a proxy is involved in the path of the request."],["FROM","Contains an Internet email address for a human user who controls the requesting user agent."],["HOST","Specifies the domain name of the server and (optionally) the TCP port number on which the server is listening."],["IF_MATCH","Makes a request conditional based on the E-Tag."],["IF_MODIFIED_SINCE","Makes a request conditional based on the modification date."],["IF_NONE_MATCH","Makes a request conditional based on the E-Tag."],["IF_RANGE","Makes a request conditional based on range."],["IF_UNMODIFIED_SINCE","Makes the request conditional based on the last modification date."],["LAST_MODIFIED","Content-Types that are acceptable for the response."],["LINK","Allows the server to point an interested client to another resource containing metadata about the requested resource."],["LOCATION","Indicates the URL to redirect a page to."],["MAX_FORWARDS","Indicates the max number of intermediaries the request should be sent through."],["ORIGIN","Indicates where a fetch originates from."],["PRAGMA","HTTP/1.0 header usually used for backwards compatibility."],["PROXY_AUTHENTICATE","Defines the authentication method that should be used to gain access to a proxy."],["PROXY_AUTHORIZATION","Contains the credentials to authenticate a user agent to a proxy server."],["PUBLIC_KEY_PINS","Associates a specific cryptographic public key with a certain server."],["PUBLIC_KEY_PINS_REPORT_ONLY","Sends reports of pinning violation to the report-uri specified in the header."],["RANGE","Indicates the part of a document that the server should return."],["REFERER","Contains the address of the previous web page from which a link to the currently requested page was followed."],["REFERRER_POLICY","Governs which referrer information should be included with requests made."],["REFRESH","Informs the web browser that the current page or frame should be refreshed."],["RETRY_AFTER","The Retry-After response HTTP header indicates how long the user agent should wait before making a follow-up request. There are two main cases this header is used:"],["SEC_WEBSOCKET_ACCEPT","The |Sec-WebSocket-Accept| header field is used in the WebSocket opening handshake. It is sent from the server to the client to confirm that the server is willing to initiate the WebSocket connection."],["SEC_WEBSOCKET_EXTENSIONS","The |Sec-WebSocket-Extensions| header field is used in the WebSocket opening handshake. It is initially sent from the client to the server, and then subsequently sent from the server to the client, to agree on a set of protocol-level extensions to use for the duration of the connection."],["SEC_WEBSOCKET_KEY","The |Sec-WebSocket-Key| header field is used in the WebSocket opening handshake. It is sent from the client to the server to provide part of the information used by the server to prove that it received a valid WebSocket opening handshake. This helps ensure that the server does not accept connections from non-WebSocket clients (e.g., HTTP clients) that are being abused to send data to unsuspecting WebSocket servers."],["SEC_WEBSOCKET_PROTOCOL","The |Sec-WebSocket-Protocol| header field is used in the WebSocket opening handshake. It is sent from the client to the server and back from the server to the client to confirm the subprotocol of the connection.  This enables scripts to both select a subprotocol and be sure that the server agreed to serve that subprotocol."],["SEC_WEBSOCKET_VERSION","The |Sec-WebSocket-Version| header field is used in the WebSocket opening handshake.  It is sent from the client to the server to indicate the protocol version of the connection.  This enables servers to correctly interpret the opening handshake and subsequent data being sent from the data, and close the connection if the server cannot interpret that data in a safe manner."],["SERVER","Contains information about the software used by the origin server to handle the request."],["SET_COOKIE","Used to send cookies from the server to the user agent."],["STRICT_TRANSPORT_SECURITY","Tells the client to communicate with HTTPS instead of using HTTP."],["TE","Informs the server of transfer encodings willing to be accepted as part of the response."],["TRAILER","Allows the sender to include additional fields at the end of chunked messages."],["TRANSFER_ENCODING","Specifies the form of encoding used to safely transfer the entity to the client."],["UPGRADE","Used as part of the exchange to upgrade the protocol."],["UPGRADE_INSECURE_REQUESTS","Sends a signal to the server expressing the client’s preference for an encrypted and authenticated response."],["USER_AGENT","Contains a string that allows identifying the requesting client's software."],["VARY","Determines how to match future requests with cached responses."],["VIA","Added by proxies to track routing."],["WARNING","General HTTP header contains information about possible problems with the status of the message."],["WWW_AUTHENTICATE","Defines the authentication method that should be used to gain access to a resource."],["X_CONTENT_TYPE_OPTIONS","Marker used by the server to indicate that the MIME types advertised in the `content-type` headers should not be changed and be followed."],["X_DNS_PREFETCH_CONTROL","Controls DNS prefetching."],["X_FRAME_OPTIONS","Indicates whether or not a browser should be allowed to render a page in a frame."],["X_XSS_PROTECTION","Stop pages from loading when an XSS attack is detected."]],"enum":[["ContentEncoding","Represents supported types of content encodings"],["Entry","A view into a single location in a `HeaderMap`, which may be vacant or occupied."]],"struct":[["Drain","A drain iterator for `HeaderMap`."],["GetAll","A view to all values stored in a single entry."],["HeaderMap","A set of HTTP headers"],["HeaderName","Represents an HTTP header field name"],["HeaderValue","Represents an HTTP header field value."],["IntoIter","An owning iterator over the entries of a `HeaderMap`."],["InvalidHeaderName","A possible error when converting a `HeaderName` from another type."],["InvalidHeaderNameBytes","A possible error when converting a `HeaderName` from another type."],["InvalidHeaderValue","A possible error when converting a `HeaderValue` from a string or byte slice."],["InvalidHeaderValueBytes","A possible error when converting a `HeaderValue` from a string or byte slice."],["Iter","`HeaderMap` entry iterator."],["Keys","An iterator over `HeaderMap` keys."],["OccupiedEntry","A view into a single occupied location in a `HeaderMap`."],["ToStrError","A possible error when converting a `HeaderValue` to a string representation."],["VacantEntry","A view into a single empty location in a `HeaderMap`."],["ValueDrain","An drain iterator of all values associated with a single header name."],["ValueIter","An iterator of all values associated with a single header name."],["ValueIterMut","A mutable iterator of all values associated with a single header name."],["Values","`HeaderMap` value iterator."]],"trait":[["AsHeaderName","A marker trait used to identify values that can be used as search keys to a `HeaderMap`."],["IntoHeaderName","A marker trait used to identify values that can be used as insert keys to a `HeaderMap`."]]});