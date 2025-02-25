package com.b44t.messenger;

import android.content.Intent;
import android.graphics.Bitmap;
import android.graphics.pdf.PdfRenderer;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.Environment;
import android.os.ParcelFileDescriptor;
import android.provider.Settings;
import android.view.WindowManager;
import android.widget.ImageView;
import android.widget.Toast;

import androidx.core.content.FileProvider;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import org.thoughtcrime.securesms.BaseActionBarActivity;
import org.thoughtcrime.securesms.PdfPageAdapter;
import org.thoughtcrime.securesms.R;

import java.io.File;
import java.io.IOException;

public class ActivityPDFViewer extends BaseActionBarActivity
{
  public static String prfFilePath = "";
  private PdfRenderer pdfRenderer;
  private ParcelFileDescriptor fileDescriptor;


  @Override
  protected void onCreate(Bundle savedInstanceState)
  {
    super.onCreate(savedInstanceState);
    setContentView(R.layout.activity_pdf_viewer);
    getWindow().setFlags(WindowManager.LayoutParams.FLAG_SECURE, WindowManager.LayoutParams.FLAG_SECURE);

    System.out.println("===path=>>"+prfFilePath);

    RecyclerView recyclerView = findViewById(R.id.pdf_recycler_view);
    recyclerView.setLayoutManager(new LinearLayoutManager(this, LinearLayoutManager.VERTICAL, false));

    try {
//      File file = new File(getFilesDir(), "delta.pdf");
      File file = new File(prfFilePath);
      if (!file.exists())
      {
        Toast.makeText(this, "PDF file not found!", Toast.LENGTH_SHORT).show();
        return;
      }

      fileDescriptor = ParcelFileDescriptor.open(file, ParcelFileDescriptor.MODE_READ_ONLY);
      pdfRenderer = new PdfRenderer(fileDescriptor);

      PdfPageAdapter adapter = new PdfPageAdapter(pdfRenderer);
      recyclerView.setAdapter(adapter);

      // Ensuring the PDF is rendered as soon as RecyclerView is set
      recyclerView.post(() -> adapter.notifyDataSetChanged());

    } catch (IOException e) {
      e.printStackTrace();
      Toast.makeText(this, "Error loading PDF file", Toast.LENGTH_SHORT).show();
    }
  }

  @Override
  protected void onDestroy() {
    super.onDestroy();
    try {
      if (pdfRenderer != null) {
        pdfRenderer.close();
      }
      if (fileDescriptor != null) {
        fileDescriptor.close();
      }
    } catch (IOException e) {
      e.printStackTrace();
    }
  }
}
