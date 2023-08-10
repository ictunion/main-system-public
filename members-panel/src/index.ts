import * as React from "react";
import * as ReactDOM from "react-dom";

import initKeycloak from './keycloak';
import App from './App';

window.addEventListener("load", () => {
    initKeycloak().then(keycloak => {
        ReactDOM.render(React.createElement(App, { keycloak }), document.getElementById('app'));
    });
});
