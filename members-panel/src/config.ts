export type Protocol = "http" | "https";
export type Url = `${Protocol}://${string}`;

export interface Config {
    keycloak_url: Url;
    keycloak_realm: string;
    keycloak_client_id: string;
}

const config = require("../config.json") as Config;
export default config;
