# Building
Use `nix develop` for seemless development without android studio.

Once the APK is ready, start the emulator, flash the APK as below:

```
# Set the paths accordin to your setup
export ANDROID_HOME=$HOME/Library/Android/sdk
export PATH=$PATH:$ANDROID_HOME/emulator
export PATH=$PATH:$ANDROID_HOME/tools
export PATH=$PATH:$ANDROID_HOME/tools/bin
export PATH=$PATH:$ANDROID_HOME/platform-tools

emulator -list-avds
emulator @Pixel_7_Pro_API_35 (@<emulator_name>)

adb install <apk path>
```

# Logcat

```
adb devices
adb -s emulator-5554 logcat
```

# Sync Changes from the Public Repository
Whenever you want to update your private repository with changes from the public repository (i.e. `upstream`), follow these steps:

Where: 
Public repo: Delta chat
Private: Privitty

*Step 1: Clone the Private Repository Locally*
```
git clone https://github.com/Privitty/priv-deltachat-android.git
cd priv-deltachat-android
git submodule update --init --recursive
```

*Step 2: Add the Public Repo as an Upstream Remote*

```
git remote add upstream https://github.com/deltachat/deltachat-android.git
git remote -v
```

*Step 3: Fetch Changes from the Public Repo*
```
git fetch upstream
```

*Step 4: Merge Upstream Changes into Your Branch*

```
git checkout main
git merge upstream/main
```
Solve all the merge conflicts is any and later `git add && git commit`

*Step 5: Push the Changes to Your Private Repo*
Push all local branches and tags to your private repository:

```
git push origin main             # Push all branches
```
