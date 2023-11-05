ALTER TABLE SubscriptionState ADD COLUMN host TEXT DEFAULT 'wss://bsky.social';
ALTER TABLE SubscriptionState ALTER COLUMN host DROP DEFAULT; 
ALTER TABLE SubscriptionState DROP CONSTRAINT subscriptionstate_service_key;
ALTER TABLE SubscriptionState ADD CONSTRAINT service_host_unique UNIQUE(service, host);
