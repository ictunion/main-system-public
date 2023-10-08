import * as React from 'react';
import Keycloak from 'keycloak-js';
import { PostgrestClient } from '@supabase/postgrest-js'

import { BrowserRouter, Routes, Route } from 'react-router-dom';
import {
    createBrowserRouter,
    createRoutesFromElements,
    RouterProvider,
  } from "react-router-dom";

import Layout from '@app/Layout';
import WelcomePage from '@app/pages/Welcome';
import NotFoundPage from '@app/pages/NotFound';
import MembersTablePage from '@app/pages/MembersTable';
import NewMemberPage from '@app/pages/NewMember';
import DetailMemberPage from '@app/pages/DetailMemberPage';
import NewMember from '@app/pages/NewMember';

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
    }

    logout() {
        this.props.keycloak.logout();
    }

    render() {   
        const router = createBrowserRouter(
            createRoutesFromElements(
                <Route path="/" element={<Layout logout={this.logout.bind(this)} />}>
                    <Route index element={<WelcomePage userInfo={this.state.userInfo} />} />
                    <Route path="members" >
                        <Route path="table" element={<MembersTablePage postgrest={this.props.postgrest} />} />
                        <Route path="new" element={<NewMemberPage postgrest={this.props.postgrest}/>} />
                        <Route path=":id" element={<DetailMemberPage postgrest={this.props.postgrest}/>} />
                    </Route>
                    <Route path="*" element={<NotFoundPage />} />
                </Route>
            )
        );

        return (
            <RouterProvider router={router} />
        );
    }
}
