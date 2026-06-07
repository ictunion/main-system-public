import { UserManager, Log, User, } from 'oidc-client-ts';

// Configure logger backend (statically);
Log.setLogger(console);

class Oidc {
    user: User | undefined;
    manager: UserManager;

    private static checkExpired(user: User) {
        if (user.expired) {
            throw ("User session expired");
        }
    }

    constructor(m: UserManager) {
        this.manager = m;

        // when user gets updated, pull it out of the manager
        this.manager.events.addUserLoaded(this.setCurrentUser.bind(this));
    }

    async authenticate(): Promise<User> {
        const currentUrl = new window.URL(window.location.href);
        let user: User;

        if (currentUrl.searchParams.has("code") && currentUrl.searchParams.has("state")) {
            user = await this.handleCallback();
        } else {
            user = await this.readPersistentSession();
        }

        // Handle expired session
        Oidc.checkExpired(user);

        this.user = user;
        return user;
    }

    // Rescript will always be calling this getter to get to tokens
    // which will ensure it gets fresh updated tokens not the stale ones
    getCurrentUser(): User | undefined {
        return this.user;
    }

    signIn(): void {
        this.manager.signinRedirect();
    }

    signOut(): void {
        this.manager.signoutRedirect();
    }

    private setCurrentUser(user: User): void {
        this.user = user;
    }

    private async handleCallback(): Promise<User> {
        // get user via manager
        const user: User = await this.manager.signinRedirectCallback();

        // cleanup url params
        window.history.replaceState({}, document.title, window.location.pathname);

        return user;
    }

    private async readPersistentSession(): Promise<User> {
        const user: User | null = await this.manager.getUser();

        if (!user) {
            throw ("No previous session");
        }

        return user;
    }
}

export default Oidc;
