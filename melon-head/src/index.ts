import { App } from './App.bs';
import { createRoot } from 'react-dom/client';
import * as React from "react";
import config from "../config.json";
import { UserManager, Log, User } from 'oidc-client-ts';

require("../static/css/main.css");

const authorityUrl = `${config.keycloak_url}/realms/${config.keycloak_realm}`;
Log.setLogger(console);

const oidcManager = new UserManager({
    authority: authorityUrl,
    client_id: config.keycloak_client_id,
    redirect_uri: `${window.location.origin}/`,
    scope: "openid profile email",
    automaticSilentRenew: true
});

async function initOidc(): Promise<User | undefined> {
    const currentUrl = new window.URL(window.location.href);

    // when redirecting from login
    if (currentUrl.searchParams.has("code") && currentUrl.searchParams.has("state")) {
        try {
            const user = await oidcManager.signinRedirectCallback();
            window.history.replaceState({}, document.title, window.location.pathname);

            if (user && !user.expired) {
                return user;
            } else {
                throw ("user expired");
            }

        } catch (error) {
            console.error("Error handling signin callback", error);
        }
    }

    // Read persistent session
    try {
        const user = await oidcManager.getUser();
        if (user && !user.expired) {
            // Session is alive and valid
            return user;
        } else {
            // No active session found, redirect to local Keycloak
            console.log("No active session. Redirecting to Keycloak...");
            await oidcManager.signinRedirect();
        }
    } catch (error) {
        console.error("Error checking session:", error);
    }
}

window.addEventListener("load", async () => {
    // create application root
    const container = document.getElementById('app');
    const root = createRoot((container as HTMLElement));

    const renderApp = (user: User) => {
        root.render(React.createElement(App.make, {
            config,
            oidcUser: user, // Pass the fresh user object reference here
            userManager: oidcManager
        }));
    };

    // init keycloak and start app
    const oidcUser = await initOidc();

    // if undefined we're waiting for login redirect - skipp the app initialization
    if (!oidcUser) return;
    renderApp(oidcUser);

    // refresh user in app after token refresh
    oidcManager.events.addUserLoaded((newUser) => {
        renderApp(newUser)
    });
});
