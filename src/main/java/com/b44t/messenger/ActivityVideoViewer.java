package com.b44t.messenger;

import android.net.Uri;
import android.os.Bundle;
import android.view.WindowManager;
import android.widget.MediaController;
import android.widget.Toast;
import android.widget.VideoView;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;


import org.thoughtcrime.securesms.R;

import java.io.File;

public class ActivityVideoViewer extends AppCompatActivity
{
    public static String prfFilePath = "";
    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState)
    {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_video_viewer);
      getWindow().setFlags(WindowManager.LayoutParams.FLAG_SECURE, WindowManager.LayoutParams.FLAG_SECURE);
      VideoView videoView = findViewById(R.id.videoView);
        System.out.println("===path=>>"+prfFilePath);
//        File vdoFile = new  File(this.getBaseContext().getFilesDir(), "abc5.mov");
      File file = new File(prfFilePath);
      if (!file.exists())
      {
        Toast.makeText(this, "PDF file not found!", Toast.LENGTH_SHORT).show();
        return;
      }
      if(file.exists())
      {
          videoView.setVideoURI(Uri.parse(file.getAbsolutePath()));
          videoView.setMediaController(new MediaController(this));
          videoView.start();
      };

    }
}
