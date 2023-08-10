import * as React from "react";
import * as ReactDOM from "react-dom";

import initKeycloak from './keycloak';
import App from './App';

window.addEventListener("load", () => {
    initKeycloak().then(k => {
        console.log(k.hasRealmRole('registration-procession'));
        ReactDOM.render(React.createElement(App, k), document.getElementById('app'));
    });
});
