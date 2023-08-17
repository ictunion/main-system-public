// Import material UI fonts
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

// import styles
import '@app/styles/index.css';

// Import App
import * as React from "react";
import * as ReactDOM from "react-dom";

import ApiAdapter from '@app/ApiAdapter';
import initKeycloak from '@app/keycloak';
import config from '@app/config';
import App from '@app/App';

window.addEventListener("load", () => {
    initKeycloak().then(keycloak => {
        const apiAdapter = new ApiAdapter(config, keycloak);
        ReactDOM.render(React.createElement(App, { keycloak, apiAdapter }), document.getElementById('app'));
    });
});
