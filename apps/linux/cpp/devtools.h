#pragma once
// Developer options (#287): what the Developer tab in Settings reads to say why a page is
// slow. Three parts, all cheap when developer options are off:
//
// - An image interceptor on the engine. Every image url QML resolves (a poster, a banner, a
//   portrait; never the icons in the resources) passes through it. With images off it hands
//   back an empty url, so nothing is read or decoded. With measuring on it sends a local file
//   to the `image://dev/` provider instead, which decodes it exactly as Image would (the
//   requested size, aspect kept, never scaled up) and records what it cost. Off, it hands the
//   url back untouched and Image takes its usual path.
// - The scene x-ray: a walk over an item tree that counts items by type, images by status,
//   decoded pixels, and the images built outside the window.
// - A monotonic clock in milliseconds that QML, Rust and the decode records share.
#include <QtCore/QObject>
#include <QtCore/QString>

class QQmlApplicationEngine;

// Installs the interceptor and the `dev` image provider. Once, before the first load.
void install_dev_tools(QQmlApplicationEngine &engine);
// measure: decodes go through the provider and are recorded. images: false stops every
// image from loading. Takes effect for urls resolved from now on, so a page built later.
void dev_set_image_flags(bool measure, bool images);
// Milliseconds since the process started, on a steady clock.
double dev_now_ms();
// JSON: { requests, blocked, decodes, decodedBytes, fileBytes, decodeMs, pending,
// list: [{ at, ms, file, bytes, srcW, srcH, w, h, thread }] } for decodes that started at
// or after `since` (ms on the dev clock), newest last, at most 400 of them.
QString dev_image_stats(double since);
// JSON: the x-ray of `item`'s tree; `window` is the QQuickWindow that tells on-screen from
// off. { items, visible, offscreen, texts, shapes, loaders, images: { total, ready,
// loading, error, none, offscreen, pixels, offscreenPixels }, types: [[name, n], ...] }
QString dev_scene(QObject *item, QObject *window);
// Asks the window to drop what its scene graph can rebuild, runs the JS collector and hands
// freed heap back to the system (glibc). Returns nothing; read memory before and after.
void dev_release(QObject *window);
