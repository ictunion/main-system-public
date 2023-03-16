CREATE TABLE registration_requests
    ( id UUID PRIMARY KEY DEFAULT UUID_GENERATE_V4()

    /*
     Data filled during registration:
     note we allow a lot of NULLs because of GDPR
    */

    -- we allow a lot of nulls because of gdpr
    , email TEXT
    , first_name TEXT
    , last_name TEXT
    , date_of_birth DATE
    , address TEXT
    , city TEXT
    -- better as text in case some valid value
    -- could start with 0 (though unlikely for cze)
    , postal_code TEXT
    -- we store international prefix like +420 so text
    , phone_number TEXT
    -- lets keep this as a free field
    -- but in the future we might also have a company table and
    -- have relations between companies and registration_requests
    , company_name TEXT
    , occupation TEXT

    /*
     Atomatically populated data
     Also can be null because we shuld also support
     adding users manually in which case we don't all the vlaues
    */
    , created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    , verification_sent_at TIMESTAMPTZ
    , registration_ip INET
    -- lets store it as text for now. eventually it might be relation?
    -- this will be an ISO 639-1 value
    , registration_local TEXT
    , registration_user_agent TEXT
    , registration_source TEXT

    /*
     Values needed for further business logic processing
    */
    , confirmation_token TEXT
    , confirmed_at TIMESTAMPTZ

    /*
     GDPR
    */
    , personal_info_wiped_at TIMESTAMPTZ
    );

-- Create useful indexes
CREATE INDEX registration_request_email ON registration_requests(email);
CREATE INDEX registration_request_city ON registration_requests(city);
CREATE INDEX registration_request_postal_code ON registration_requests(postal_code);
CREATE INDEX registration_request_company_name ON registration_requests(company_name);
CREATE INDEX registration_request_created_at ON registration_requests(created_at);
CREATE INDEX registration_request_registration_local ON registration_requests(registration_local);
CREATE INDEX registration_request_registration_source ON registration_requests(registration_source);
CREATE UNIQUE INDEX registration_request_confirmation_token ON registration_requests(confirmation_token);
CREATE INDEX registration_request_confirmed_at ON registration_requests(confirmed_at);

-- Add SQL comments
COMMENT ON TABLE registration_requests IS 'Join requsts from website form';
COMMENT ON COLUMN registration_requests.email IS 'Null means user submitted GDPR request to remove or personal information';
COMMENT ON COLUMN registration_requests.phone_number IS 'Phone number prefixed by code like +420 666 777 888';
COMMENT ON COLUMN registration_requests.company_name IS 'Company name filled in during registration. This is free text that we can later normalize further';
COMMENT ON COLUMN registration_requests.registration_local IS 'ISO 639-1 value of localization used during registration';
COMMENT ON COLUMN registration_requests.registration_source IS 'Text field with identifier of source of registation';
COMMENT ON COLUMN registration_requests.confirmation_token IS 'Value used for email confirmation logic';
COMMENT ON COLUMN registration_requests.confirmed_at IS 'Time of email confirmation. Null mean user never confirmed their email';
COMMENT ON COLUMN registration_requests.personal_info_wiped_at IS 'Time when GDPR request to remove all personal information was processed';

-- Grant access to orca user
GRANT SELECT, INSERT, UPDATE ON TABLE registration_requests TO orca;

