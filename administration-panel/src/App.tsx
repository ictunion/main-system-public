import * as React from "react";
import Keycloak from 'keycloak-js';
import { accountSettingsUrl } from '@app/keycloak';
import ApiAdapter from '@app/ApiAdapter';
import {
    Button, Box, AppBar, Toolbar, IconButton, Typography, Grid
} from '@mui/material';
import { Menu as MenuIcon } from '@mui/icons-material';

interface UserInfo {
    name: string,
    email: string,
    preferred_username: string,
    given_name: string,
    family_name: string,
    locale: "cs" | "en",
}

interface AppState {
    userInfo: UserInfo | null;
}

interface Props {
    keycloak: Keycloak;
    apiAdapter: ApiAdapter;
}

export default class App extends React.Component<Props, AppState> {
    constructor(props: Props) {
        super(props);

        this.state = {
            userInfo: null
        };

        props.keycloak.loadUserInfo().then((userInfo: UserInfo) => {
            this.setState((state) => {
                return {
                    ...state, userInfo: userInfo
                };
            });
        });

        this.get_all_members();
    }

    logout() {
        this.props.keycloak.logout();
    }

    async get_all_members() {
        const response = await this.props.apiAdapter.get("members");
        const json = await response.json();
        console.log(json);
    }

    render() {
        return (
            <div style={{ minHeight: "100vh" }}>
                <AppBar position="static">
                    <Toolbar>
                        <IconButton
                            edge="start"
                            color="inherit"
                            aria-label="menu"
                            sx={{ mr: 2 }}>
                            <MenuIcon />
                        </IconButton>
                        <Typography variant="h6" color="inherit" component="div" sx={{ flexGrow: 1 }}>
                            Administration Panel
                        </Typography>
                        <Button color="inherit" onClick={this.logout.bind(this)}>Logout</Button>
                    </Toolbar>
                </AppBar>
                <Box sx={{ flexGrow: 1 }}>
                    <Box sx={{ flexGrow: 1 }}>
                        Hello <strong>{this.state.userInfo && this.state.userInfo.preferred_username}!</strong><br />
                        <a href={accountSettingsUrl} target="_blank">Profile Settings</a>
                    </Box>
                    <pre>
                        {JSON.stringify(this.state.userInfo)}
                    </pre>
                </Box>
            </div>
        );
    }
}
