CREATE KEYSPACE chat WITH replication = {'class': 'SimpleStrategy', 'replication_factor': 1};

USE chat;

CREATE TABLE messages
(
    channel_id uuid,
    bucket_id  int,
    message_id timeuuid,
    sender_id  uuid,
    content    text,
    PRIMARY KEY ((channel_id, bucket_id), message_id)
) WITH CLUSTERING ORDER BY (message_id DESC);