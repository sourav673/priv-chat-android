package org.thoughtcrime.securesms;

import android.content.Context;
import android.graphics.Bitmap;
import android.graphics.Matrix;
import android.graphics.pdf.PdfRenderer;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ImageView;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

public class PdfPageAdapter extends RecyclerView.Adapter<PdfPageAdapter.PdfPageViewHolder> {

    private final PdfRenderer pdfRenderer;

    public PdfPageAdapter(PdfRenderer pdfRenderer) {
        this.pdfRenderer = pdfRenderer;
    }

    @NonNull
    @Override
    public PdfPageViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext()).inflate(R.layout.pdf_page_item, parent, false);
        return new PdfPageViewHolder(view, parent.getContext());
    }

    @Override
    public void onBindViewHolder(@NonNull PdfPageViewHolder holder, int position) {
        PdfRenderer.Page page = pdfRenderer.openPage(position);

        Bitmap bitmap = Bitmap.createBitmap(page.getWidth(), page.getHeight(), Bitmap.Config.ARGB_8888);
        page.render(bitmap, null, null, PdfRenderer.Page.RENDER_MODE_FOR_DISPLAY);

        holder.bind(bitmap, position, pdfRenderer.getPageCount());
        page.close();
    }

    @Override
    public int getItemCount() {
        return pdfRenderer.getPageCount();
    }

    public static class PdfPageViewHolder extends RecyclerView.ViewHolder {
        private final ImageView pageImageView;
        private final TextView pageNumberTextView;

        private final Matrix matrix = new Matrix();
        private float scaleFactor = 1.0f;
        private float translateX = 0, translateY = 0;

        public PdfPageViewHolder(@NonNull View itemView, Context context) {
            super(itemView);
            pageImageView = itemView.findViewById(R.id.page_image);
            pageImageView.setScaleType(ImageView.ScaleType.MATRIX);

            pageNumberTextView = itemView.findViewById(R.id.page_number_text);


            // Zoom buttons
            itemView.findViewById(R.id.zoom_in_button).setOnClickListener(v -> zoomIn());
            itemView.findViewById(R.id.zoom_out_button).setOnClickListener(v -> zoomOut());

            // Movement buttons for image after zooming
            itemView.findViewById(R.id.move_left_button).setOnClickListener(v -> moveLeft());
            itemView.findViewById(R.id.move_right_button).setOnClickListener(v -> moveRight());
            itemView.findViewById(R.id.move_up_button).setOnClickListener(v -> moveUp());
            itemView.findViewById(R.id.move_down_button).setOnClickListener(v -> moveDown());
        }

        private void zoomIn() {
            scaleFactor *= 1.2f; // Zoom in factor
            updateMatrix();
        }

        private void zoomOut() {
            scaleFactor /= 1.2f; // Zoom out factor
            updateMatrix();
        }

        // Update matrix for zoom and movement
        private void updateMatrix() {


            System.out.println(scaleFactor+" == "+translateX+" X "+ translateY);
            matrix.reset();
            matrix.setScale(scaleFactor, scaleFactor);
            matrix.postTranslate(translateX, translateY); // Apply translation
            pageImageView.setImageMatrix(matrix);
        }

        private void moveLeft() {
            translateX -= 20; // Move left by 20 pixels
            updateMatrix();
        }

        private void moveRight() {
            translateX += 20; // Move right by 20 pixels
            updateMatrix();
        }

        private void moveUp() {
            translateY -= 20; // Move up by 20 pixels
            updateMatrix();
        }

        private void moveDown() {
            translateY += 20; // Move down by 20 pixels
            updateMatrix();
        }

        public void bind(Bitmap bitmap, int position, int totalPages) {
            pageImageView.setImageBitmap(bitmap);
            pageNumberTextView.setText("Page " + (position + 1) + "/" + totalPages);
            matrix.reset();

            // Calculate the scale factor based on the available space and the image's aspect ratio
            float scale = calculateFitScale(bitmap);
            matrix.setScale(scale, scale);
            pageImageView.setImageMatrix(matrix);
        }

        private float calculateFitScale(Bitmap bitmap) {
            // Get the width and height of the ImageView
            float imageViewWidth = pageImageView.getWidth();
            float imageViewHeight = pageImageView.getHeight();

            // Get the image's original width and height
            float bitmapWidth = bitmap.getWidth();
            float bitmapHeight = bitmap.getHeight();

            // Calculate the scaling factor to fit the image inside the ImageView
            float scaleX = imageViewWidth / bitmapWidth;
            float scaleY = imageViewHeight / bitmapHeight;

            // Use the smaller scale factor to avoid distortion, ensuring the image fits inside the ImageView
            return Math.max(scaleX, scaleY);
        }
    }
}
