import * as React from 'react';
import Keycloak from 'keycloak-js';
import ApiAdapter from '@app/ApiAdapter';

import {
    Button, Box, AppBar, Toolbar, IconButton, Typography, Grid
} from '@mui/material';
import { Menu as MenuIcon } from '@mui/icons-material';
import { PostgrestClient } from '@supabase/postgrest-js'

import { BrowserRouter, Routes, Route } from 'react-router-dom';

import Layout from '@app/Layout';
import WelcomePage from '@app/pages/Welcome';
import NotFoundPage from '@app/pages/NotFound';

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
    postgrest: PostgrestClient;
}

export default class App extends React.Component<Props, AppState> {
    state: AppState = {
        userInfo: null,
    }

    componentDidMount() {
        this.props.keycloak.loadUserInfo().then((userInfo: UserInfo) => {
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
        const response = await this.props.postgrest.from('members').select()
        const data = response.data;
        console.log(data)
    }

    render() {
        return (
            <BrowserRouter>
                <Routes>
                    <Route path="/" element={<Layout logout={this.logout.bind(this)} />}>
                        <Route index element={<WelcomePage userInfo={this.state.userInfo} />} />
                        <Route path="*" element={<NotFoundPage />} />
                    </Route>
                </Routes>
            </BrowserRouter>
        );
    }
}
