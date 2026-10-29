// The Mac's Now Playing centre and media keys (#187), behind the Player.mpris* seam MPRIS
// fills on Linux. src/macnp.rs is the Rust half and the only caller, always on the Qt
// thread, which on the Mac is the main thread MediaPlayer wants. The remote command
// handlers run wherever Apple runs them and only call back into Rust, which queues the
// page's signal. Built on its own with ARC, see build.rs.
//
// AniBeam holds the keys only while a player page is up: the targets go on with the first
// Playing or Paused and all come off with the Stopped the page sends as it goes, the release
// IINA's rule asks for. ANIBEAM_NP_TRACE=1 prints each publish, elapsed write, release and
// command.

#import <AppKit/AppKit.h>
#import <MediaPlayer/MediaPlayer.h>

#include <atomic>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <cstring>

typedef void (*anibeam_np_command)(const char *name, double value);

namespace {

std::atomic<anibeam_np_command> command_cb{nullptr};
// [command, token] pairs: removeTarget: takes both, and holding the token is what keeps a
// target alive, as addTargetWithHandler: asks. nil while nothing plays.
NSMutableArray<NSArray *> *targets = nil;
// The same for the 15 s skips, which come and go on their own (see offer_skips).
NSMutableArray<NSArray *> *skips = nil;
// The dictionary last given to the centre, so a seek or a jump rewrites one key of it.
NSMutableDictionary<NSString *, id> *published = nil;
// The poster URL last sent and what came of it, nil when it would not load, so a missing
// or broken file is read once rather than at every publish.
NSString *art_source = nil;
MPMediaItemArtwork *artwork = nil;

bool trace() {
    static const bool on = [] {
        const char *v = std::getenv("ANIBEAM_NP_TRACE");
        return v && std::strcmp(v, "1") == 0;
    }();
    return on;
}

void relay(const char *name, double value) {
    if (trace()) std::fprintf(stderr, "anibeam: now playing: command %s %g\n", name, value);
    if (anibeam_np_command cb = command_cb.load()) cb(name, value);
}

void add(NSMutableArray<NSArray *> *into, MPRemoteCommand *command,
         void (^act)(MPRemoteCommandEvent *)) {
    command.enabled = YES;
    id token = [command addTargetWithHandler:^MPRemoteCommandHandlerStatus(MPRemoteCommandEvent *event) {
        act(event);
        return MPRemoteCommandHandlerStatusSuccess;
    }];
    if (token) [into addObject:@[command, token]];
}

void add(MPRemoteCommand *command, void (^act)(MPRemoteCommandEvent *)) { add(targets, command, act); }

void add(MPRemoteCommand *command, const char *name) {
    add(command, ^(MPRemoteCommandEvent *) { relay(name, 0); });
}

void remove_all(NSArray<NSArray *> *pairs) {
    for (NSArray *pair in pairs) [(MPRemoteCommand *)pair[0] removeTarget:pair[1]];
}

// Control Centre draws the 15 s skips in place of previous and next whenever they are on,
// so they are offered only to an episode with no neighbour either way, and an episode that
// has one gets previous and next, as MPRIS gives it on Linux. Their targets come off as well
// as `enabled`: a command that still has a target can be drawn dimmed in that slot, and with
// none the system has nothing to offer. Skips are 15 s, the spec's default, since AniBeam has
// no seek preference.
void offer_skips(bool on) {
    if (on == (skips != nil)) return;
    MPRemoteCommandCenter *rc = MPRemoteCommandCenter.sharedCommandCenter;
    if (!on) {
        remove_all(skips);
        skips = nil;
        rc.skipForwardCommand.enabled = NO;
        rc.skipBackwardCommand.enabled = NO;
        return;
    }
    skips = [NSMutableArray array];
    rc.skipForwardCommand.preferredIntervals = @[ @15 ];
    rc.skipBackwardCommand.preferredIntervals = @[ @15 ];
    add(skips, rc.skipForwardCommand, ^(MPRemoteCommandEvent *e) {
        relay("seek", ((MPSkipIntervalCommandEvent *)e).interval);
    });
    add(skips, rc.skipBackwardCommand, ^(MPRemoteCommandEvent *e) {
        relay("seek", -((MPSkipIntervalCommandEvent *)e).interval);
    });
}

// The video player's set, in the names the page already takes from MPRIS. The skips follow
// the episode, see offer_skips.
void hold_keys() {
    if (targets) return;
    targets = [NSMutableArray array];
    MPRemoteCommandCenter *rc = MPRemoteCommandCenter.sharedCommandCenter;
    add(rc.playCommand, "play");
    add(rc.pauseCommand, "pause");
    add(rc.togglePlayPauseCommand, "playPause");
    add(rc.stopCommand, "stop");
    add(rc.nextTrackCommand, "next");
    add(rc.previousTrackCommand, "previous");
    add(rc.changePlaybackPositionCommand, ^(MPRemoteCommandEvent *e) {
        relay("setPosition", ((MPChangePlaybackPositionCommandEvent *)e).positionTime);
    });
}

void release_keys() {
    if (!targets && !published) return;
    MPNowPlayingInfoCenter *center = MPNowPlayingInfoCenter.defaultCenter;
    center.playbackState = MPNowPlayingPlaybackStateStopped;
    center.nowPlayingInfo = nil;
    remove_all(targets);
    targets = nil;
    offer_skips(false);
    published = nil;
    art_source = nil;
    artwork = nil;
    if (trace()) std::fprintf(stderr, "anibeam: now playing: released\n");
}

// The poster scaled down to fit the size asked for, never up. The image is drawn only when
// the system draws the result, so the handler stays cheap on whatever thread calls it.
NSImage *fit(NSImage *image, CGSize bounds) {
    NSSize have = image.size;
    if (have.width <= bounds.width && have.height <= bounds.height) return image;
    CGFloat k = MIN(bounds.width / have.width, bounds.height / have.height);
    NSSize size = NSMakeSize(std::floor(have.width * k), std::floor(have.height * k));
    return [NSImage imageWithSize:size
                          flipped:NO
                   drawingHandler:^BOOL(NSRect rect) {
                       [image drawInRect:rect];
                       return YES;
                   }];
}

// The page sends the poster as a file:// URL. A missing or unreadable file is no artwork;
// either answer is kept for as long as the URL stays the same.
MPMediaItemArtwork *artwork_for(const char *url) {
    NSString *u = url && *url ? [NSString stringWithUTF8String:url] : nil;
    if (!u) return nil;
    if ([u isEqualToString:art_source]) return artwork;
    art_source = u;
    artwork = nil;
    NSURL *file = [NSURL URLWithString:u];
    NSImage *image = file ? [[NSImage alloc] initWithContentsOfURL:file] : nil;
    if (!image || image.size.width <= 0 || image.size.height <= 0) return nil;
    artwork = [[MPMediaItemArtwork alloc] initWithBoundsSize:image.size
                                              requestHandler:^NSImage *(CGSize size) {
                                                  return fit(image, size);
                                              }];
    return artwork;
}

NSString *ns_string(const char *s) {
    NSString *out = s ? [NSString stringWithUTF8String:s] : nil;
    return out ?: @"";
}

}  // namespace

extern "C" void anibeam_np_set_command(anibeam_np_command cb) { command_cb.store(cb); }

extern "C" void anibeam_np_publish(int32_t status, const char *title, const char *artist,
                                   const char *art_url, double duration, double elapsed,
                                   double rate, bool can_next, bool can_prev) {
    if (status != 1 && status != 2) {
        release_keys();
        return;
    }
    bool playing = status == 1;
    hold_keys();
    MPRemoteCommandCenter *rc = MPRemoteCommandCenter.sharedCommandCenter;
    rc.nextTrackCommand.enabled = can_next;
    rc.previousTrackCommand.enabled = can_prev;
    bool skipping = !can_next && !can_prev;
    offer_skips(skipping);
    rc.changePlaybackPositionCommand.enabled = duration > 0;

    NSMutableDictionary<NSString *, id> *info = [NSMutableDictionary dictionary];
    info[MPMediaItemPropertyTitle] = ns_string(title);
    info[MPMediaItemPropertyArtist] = ns_string(artist);
    info[MPMediaItemPropertyPlaybackDuration] = @(duration);
    info[MPNowPlayingInfoPropertyMediaType] = @(MPNowPlayingInfoMediaTypeVideo);
    info[MPNowPlayingInfoPropertyElapsedPlaybackTime] = @(elapsed);
    info[MPNowPlayingInfoPropertyPlaybackRate] = @(rate);
    info[MPNowPlayingInfoPropertyDefaultPlaybackRate] = @1.0;
    MPMediaItemArtwork *art = artwork_for(art_url);
    if (art) info[MPMediaItemPropertyArtwork] = art;

    MPNowPlayingInfoCenter *center = MPNowPlayingInfoCenter.defaultCenter;
    published = info;
    center.nowPlayingInfo = [info copy];
    center.playbackState = playing ? MPNowPlayingPlaybackStatePlaying : MPNowPlayingPlaybackStatePaused;
    if (trace())
        std::fprintf(stderr, "anibeam: now playing: %s \"%s\" at %.1f of %.1f s, rate %g, art %s, skips %s\n",
                     playing ? "playing" : "paused", title ? title : "", elapsed, duration, rate,
                     art ? "yes" : "no", skipping ? "yes" : "no");
}

extern "C" void anibeam_np_elapsed(double elapsed) {
    if (!published) return;
    published[MPNowPlayingInfoPropertyElapsedPlaybackTime] = @(elapsed);
    MPNowPlayingInfoCenter.defaultCenter.nowPlayingInfo = [published copy];
    if (trace()) std::fprintf(stderr, "anibeam: now playing: elapsed %.1f s\n", elapsed);
}
