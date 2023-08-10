import * as React from "react";
import Keycloak from 'keycloak-js';
import { accountSettingsUrl } from './keycloak';

interface UserInfo {
    name: string,
    email: string,
    preferred_username: string,
    given_name: string,
    family_name: string,
    locale: "cs" | "en",
}

interface AppState {
    userInfo: UserInfo;
}

export default class App extends React.Component<Keycloak, AppState> {
    constructor(props: Keycloak) {
        super(props);

        this.state = {
            userInfo: null
        };

        props.loadUserInfo().then((userInfo: UserInfo) => {
            this.setState((state) => {
                return {
                    ...state, userInfo: userInfo
                };
            });
        });
    }

    logout() {
        this.props.logout();
    }

    render() {
        return (
            <div>
                Hello <strong>{this.state.userInfo && this.state.userInfo.preferred_username}!</strong><br />
                <a href={accountSettingsUrl} target="_blank">Profile Settings</a>
                <pre>
                    {JSON.stringify(this.state.userInfo)}
                </pre>

                <button onClick={this.logout.bind(this)}>Logout</button>
            </div>
        );
    }
}
