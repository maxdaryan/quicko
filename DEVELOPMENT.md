# Android Architecture & UniFFI Status

## Achievements
- **Android Project Scaffolded**: `ui-android/` contains a modern Jetpack Compose + MVVM application.
- **UniFFI Proc-Macro Migration**: Successfully migrated to `uniffi::export` approach. All types are annotated and the FFI crate compiles cleanly.
- **Kotlin Bindings Generated**: Bindings are generated in `dev.quicko.core` package using the host dylib.
- **ViewModel & UI Enhanced**: `QuickoViewModel.kt` and `MainActivity.kt` are fully wired with session and key management features.

## Resolved Issues
- **UniFFI UDL Parser (0.28.3)**: Fixed by migrating to the proc-macro approach, removing the dependency on the fragile UDL parser.

## Pending Tasks
1. Run `scripts/build_android.sh` in an environment with the Android NDK installed to compile for Android targets.
2. Implement Message sending/receiving logic in the ViewModel.
3. Add persistence for QuickoKeys on Android (EncryptedSharedPreferences).
