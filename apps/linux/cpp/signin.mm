// The Mac's tracker sign-in window (#1176): the authorize page in an
// ASWebAuthenticationSession, a window of its own rather than a tab. macOS lets the default
// browser serve the session when it can (Zen did, on the machine this was tried on), else
// Safari does, so the user's existing sign-in to the tracker carries over either way.
// src/signin.rs is the Rust half and the only caller, always on the Qt thread, which on the
// Mac is the main thread AppKit wants.
// Built with ARC beside cpp/nowplaying.mm, see build.rs.
//
// The core's redirect is the plain http loopback its own listener answers, never a custom
// scheme, so the session never completes by itself: the page redirects to 127.0.0.1, the
// core's listener takes the code, and the shell closes the window once the connect job
// ends. The scheme given here is one nothing ever redirects to.

#import <AppKit/AppKit.h>
#import <AuthenticationServices/AuthenticationServices.h>

#include <cstdio>

// The window the session's sheet hangs from: the app's own, the one Qt shows.
@interface AnibeamSignInAnchor : NSObject <ASWebAuthenticationPresentationContextProviding>
@end

@implementation AnibeamSignInAnchor
- (ASPresentationAnchor)presentationAnchorForWebAuthenticationSession:(ASWebAuthenticationSession *)session {
    (void)session;
    NSWindow *window = NSApp.mainWindow ?: NSApp.keyWindow;
    if (!window) {
        for (NSWindow *w in NSApp.windows) {
            if (w.visible) { window = w; break; }
        }
    }
    return window ?: [[NSWindow alloc] init];
}
@end

namespace {

// The session up now and its anchor, which the session only holds weakly. nil while no
// sign-in window is open.
ASWebAuthenticationSession *session = nil;
AnibeamSignInAnchor *anchor = nil;

NSString *const kNeverScheme = @"anibeam-signin";

void close_session() {
    ASWebAuthenticationSession *s = session;
    session = nil;
    // A cancel calls the completion handler, which finds the session already gone.
    [s cancel];
}

} // namespace

extern "C" {

// Opens the sign-in window on `url` (UTF-8), closing one already open first. False when the
// session could not start, so the caller hands the link to the browser instead.
bool anibeam_signin_start(const char *url) {
    @autoreleasepool {
        close_session();
        if (!url) return false;
        NSURL *target = [NSURL URLWithString:[NSString stringWithUTF8String:url]];
        if (!target) return false;

        // Weak: the session holds this handler, so a strong reference back would leak both.
        __block __weak ASWebAuthenticationSession *started = nil;
        ASWebAuthenticationSessionCompletionHandler done = ^(NSURL *callback, NSError *error) {
            (void)callback;
            // The user closed the window, or the shell did. Either way the core's job is
            // what decides how the sign-in ended; a window closed by hand leaves it waiting,
            // as a closed browser tab does, and the connect sheet can open it again.
            if (error && error.code != ASWebAuthenticationSessionErrorCodeCanceledLogin)
                std::fprintf(stderr, "anibeam: sign-in window: %s\n",
                             error.localizedDescription.UTF8String);
            if (session == started) session = nil;
        };
        ASWebAuthenticationSession *s = nil;
        if (@available(macOS 14.4, *)) {
            ASWebAuthenticationSessionCallback *cb =
                [ASWebAuthenticationSessionCallback callbackWithCustomScheme:kNeverScheme];
            s = [[ASWebAuthenticationSession alloc] initWithURL:target callback:cb completionHandler:done];
        } else {
            s = [[ASWebAuthenticationSession alloc] initWithURL:target
                                              callbackURLScheme:kNeverScheme
                                              completionHandler:done];
        }
        if (!s) return false;
        started = s;
        if (!anchor) anchor = [[AnibeamSignInAnchor alloc] init];
        s.presentationContextProvider = anchor;
        // The browser's cookies, so a provider already signed in there only asks to allow.
        s.prefersEphemeralWebBrowserSession = NO;
        session = s;
        if (![s start]) {
            session = nil;
            return false;
        }
        return true;
    }
}

// Closes the sign-in window if one is open.
void anibeam_signin_close(void) {
    @autoreleasepool {
        close_session();
    }
}

} // extern "C"
