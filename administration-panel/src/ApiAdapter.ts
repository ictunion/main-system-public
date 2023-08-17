import { Config, Url } from '@app/config';
import Keycloak from 'keycloak-js';

export default class ApiAdapter {
    url: Url;
    keycloak: Keycloak;

    constructor(config: Config, keycloak: Keycloak) {
        this.url = config.api_url;
        this.keycloak = keycloak;
    }

    get(path: string): Promise<Response> {
        const url = `${this.url}/${path}`;
        return fetch(url, {
            headers: {
                "Accept": "application/json",
                "Content-Type": "application/json",
                "Authorization": `Bearer ${this.keycloak.token}`,
            },
        })
    }
}
