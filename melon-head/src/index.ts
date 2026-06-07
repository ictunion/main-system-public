import { App } from './App.bs';
import { createRoot } from 'react-dom/client';
import * as React from "react";
import config from "../config.json";
import { UserManager, User, WebStorageStateStore } from 'oidc-client-ts';
import Oidc from "./Oidc";

require("../static/css/main.css");

const authorityUrl = `${config.keycloak_url}/realms/${config.keycloak_realm}`;

const oidcManager = new UserManager({
    authority: authorityUrl,
    client_id: config.keycloak_client_id,
    redirect_uri: `${window.location.origin}/`,
    scope: "openid profile email",
    automaticSilentRenew: true,
    // Use local storage instead of default session storage
    // This is to support workflows with multiple tabs.
    //
    // Unfortunetely cookies are not supported by the library
    // which is a bummer because localStorage has more relaxed origin policy and
    // data from it could be extracted by any script which means in case of XSS they can get compromised.
    // Ideally  we would keep session storage and make sure keycloak automatically authorizes the 2nd tab without requiring login there.
    // This is the case with developement keycloak but unfortunetely it's NOT the case with production instance which always requires new login
    userStore: new WebStorageStateStore({ store: window.localStorage })
});

window.addEventListener("load", async () => {
    // create application root
    const container = document.getElementById('app');
    const root = createRoot((container as HTMLElement));
    const oidc = new Oidc(oidcManager);

    // authenticate via oidc
    try {
        await oidc.authenticate();

        // start application
        root.render(React.createElement(App.make, {
            config,
            oidc,
        }));
    } catch (error) {
        console.error("Initialization error", error);
        oidc.signIn();
    }
});
