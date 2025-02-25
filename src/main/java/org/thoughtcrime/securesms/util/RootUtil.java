package org.thoughtcrime.securesms.util;

import android.content.Context;
import android.content.pm.ApplicationInfo;
import android.content.pm.PackageManager;
import android.provider.Settings;

import com.scottyab.rootbeer.RootBeer;

import java.io.BufferedReader;
import java.io.File;
import java.io.FileReader;
import java.io.IOException;
import java.io.InputStreamReader;

public class RootUtil
{
  public static boolean isDeviceRooted()
  {
    return checkRootMethod1() || checkRootMethod2() || checkRootMethod3();
  }

  public static boolean isDeveloperModeEnabled(Context context)
  {
    try {
      return Settings.Global.getInt(
        context.getContentResolver(),
        Settings.Global.DEVELOPMENT_SETTINGS_ENABLED,
        0
      ) == 1;
    } catch (Exception e) {
      e.printStackTrace();
      return false;
    }
  }

  public static boolean isUsbDebuggingEnabled(Context context) {
    return Settings.Secure.getInt(
      context.getContentResolver(),
      Settings.Secure.ADB_ENABLED,
      0
    ) == 1;
  }

  public static boolean isDebuggable(Context context) {
    return (context.getApplicationInfo().flags & ApplicationInfo.FLAG_DEBUGGABLE) != 0;
  }

  public static boolean isRootManagementAppInstalled(Context context) {
    String[] rootManagementApps = {
      "eu.chainfire.supersu",  // SuperSU
      "com.koushikdutta.superuser",  // Superuser
      "com.thirdparty.superuser",  // Other Superuser
      "com.topjohnwu.magisk"  // Magisk
    };

    PackageManager pm = context.getPackageManager();
    for (String app : rootManagementApps) {
      try {
        pm.getPackageInfo(app, PackageManager.GET_ACTIVITIES);
        return true;
      } catch (PackageManager.NameNotFoundException ignored) {
      }
    }
    return false;
  }

  // Root detection methods

  private static boolean checkRootMethod1() {
    String buildTags = android.os.Build.TAGS;
    return buildTags != null && buildTags.contains("test-keys");
  }

  public static boolean checkRootMethod2() {
    String[] paths = {
      "/system/app/Superuser.apk", "/sbin/su", "/system/bin/su", "/system/xbin/su", "/data/local/xbin/su",
      "/data/local/bin/su", "/system/sd/xbin/su", "/system/bin/failsafe/su", "/data/local/su", "/su/bin/su"
    };
    for (String path : paths) {
      if (new File(path).exists()) return true;
    }
    return false;
  }

  private static boolean checkRootMethod3() {
    Process process = null;
    try {
      process = Runtime.getRuntime().exec(new String[]{"/system/xbin/which", "su"});
      BufferedReader in = new BufferedReader(new InputStreamReader(process.getInputStream()));
      return in.readLine() != null;
    } catch (Throwable t) {
      return false;
    } finally {
      if (process != null) process.destroy();
    }
  }

  public static boolean checkForRootProcesses() {
    String[] processes = {"supersu", "magisk", "superuser"};
    try {
      Process proc = Runtime.getRuntime().exec("ps");
      BufferedReader reader = new BufferedReader(new InputStreamReader(proc.getInputStream()));
      String line;
      while ((line = reader.readLine()) != null) {
        for (String process : processes) {
          if (line.contains(process)) {
            return true;
          }
        }
      }
    } catch (IOException ignored) {
    }
    return false;
  }

  public static boolean checkForModifiedBuildProps() {
    String[] modifiedProps = {"ro.build.tags", "ro.build.type"};
    try {
      Process process = Runtime.getRuntime().exec("getprop");
      BufferedReader reader = new BufferedReader(new InputStreamReader(process.getInputStream()));
      String line;
      while ((line = reader.readLine()) != null) {
        for (String prop : modifiedProps) {
          if (line.startsWith(prop)) {
            if (line.contains("test-keys") || line.contains("eng")) {
              return true;
            }
          }
        }
      }
    } catch (IOException ignored) {
    }
    return false;
  }

  public static boolean checkForSuspiciousSharedObjects() {
    String mapsPath = "/proc/self/maps";
    try (BufferedReader reader = new BufferedReader(new FileReader(mapsPath))) {
      String line;
      while ((line = reader.readLine()) != null) {
        if (line.contains("suspicious.so") || line.contains("suspicious.jar")) {
          return true;
        }
      }
    } catch (IOException ignored) {
    }
    return false;
  }

  public static boolean checkSuBinary() {
    String[] suPaths = {"/system/bin/su", "/system/xbin/su", "/system/sd/xbin/su"};
    for (String path : suPaths) {
      if (new File(path).exists()) {
        return true;
      }
    }
    return false;
  }

  public static boolean checkForUnusualFiles() {
    String[] unusualFiles = {"/system/bin/.su", "/system/xbin/.su"};
    for (String file : unusualFiles) {
      if (new File(file).exists()) {
        return true;
      }
    }
    return false;
  }
  public static boolean isSecured(Context context)
  {
    RootBeer rootBeer = new RootBeer(context);
    return !(
      RootUtil.isDeviceRooted() ||
        isDeveloperModeEnabled(context) ||
        rootBeer.isRooted() ||
        rootBeer.isRootedWithBusyBoxCheck() ||
        RootUtil.checkForSuspiciousSharedObjects() ||
        RootUtil.isDebuggable(context) ||
        RootUtil.isUsbDebuggingEnabled(context) ||
        RootUtil.isRootManagementAppInstalled(context) ||
        RootUtil.checkRootMethod2() ||
        RootUtil.checkForRootProcesses() ||
        RootUtil.checkForModifiedBuildProps() ||
        RootUtil.checkSuBinary() ||
        RootUtil.checkForUnusualFiles()
    );
  }
}


