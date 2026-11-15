-- Transport errors used to carry the full request URL, and a Torznab feed
-- URL holds the indexer's API key in its query, so a failed feed fetch put
-- the key into the stored activity log. The errors no longer carry URLs;
-- this drops the events written before that.
DELETE FROM events WHERE message LIKE '%apikey=%' OR body LIKE '%apikey=%';
