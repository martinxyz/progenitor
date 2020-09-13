import App from './App.svelte';

import './global.scss'
// import '@fortawesome/fontawesome-free/scss/fontawesome.scss'  // not working
import '@fortawesome/fontawesome-free/css/all.css'

const app = new App({
    target: document.body,
    props: {
    }
});

export default app;
