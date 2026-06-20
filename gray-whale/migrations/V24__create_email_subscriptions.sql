CREATE TABLE email_subscriptions
	 ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()
     -- incremental number that is also used as variable symbol for payments
	 , member_id UUID REFERENCES members(id)
	 , list TEXT NOT NULL	 
	 , listmonk_status TEXT NOT NULL
	 , listmonk_id INTEGER
	 , created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
     , updated_at TIMESTAMPTZ
	 );

create unique index email_subscriptions_member_id_list on email_subscriptions (member_id, list);

GRANT SELECT, INSERT, UPDATE, DELETE ON TABLE email_subscriptions TO orca;
