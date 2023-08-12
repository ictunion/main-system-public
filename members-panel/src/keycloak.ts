import Keycloak from 'keycloak-js';
import config, { Url } from './config';

// Initialize singleton
const keycloak: Keycloak = new Keycloak({
    url: config.keycloak_url,
    realm: config.keycloak_realm,
    clientId: config.keycloak_client_id,
});

async function initKeycloak(): Promise<Keycloak> {
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

export const accountSettingsUrl: Url = `${config.keycloak_url}/realms/${config.keycloak_realm}/account`;

export default initKeycloak;
