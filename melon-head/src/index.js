import { App } from './App.bs';
import Keycloak from 'keycloak-js';
import { createRoot } from 'react-dom/client';
import * as React from "react";
const config = require("../config.json");

const keycloak = new Keycloak({
    url: config.keycloak_url,
    realm: config.keycloak_realm,
    clientId: config.keycloak_client_id,
});

async function initKeycloak() {
    try {
        const authenticated = await keycloak.init({
            onLoad: 'login-required',
            checkLoginIframe: false
        });

        console.log(`User is ${authenticated ? 'authenticated' : 'not authenticated'}`);
    } catch (error) {
        console.error('Failed to initialize adapter:', error);
    }

    return keycloak;
}

window.addEventListener("load", async () => {
    // create application root
    const container = document.getElementById('app');
    const root = createRoot(container);

    // init keycloak and start app
    const keycloak = await initKeycloak();
    root.render(React.createElement(App.make, { keycloak, config }));
});
