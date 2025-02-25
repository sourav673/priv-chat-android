package org.thoughtcrime.securesms.customui;

import android.content.Context;
import android.graphics.Matrix;
import android.graphics.PointF;
import android.util.AttributeSet;
import android.view.MotionEvent;
import android.view.ScaleGestureDetector;
import android.view.View;

import androidx.appcompat.widget.AppCompatImageView;

public class ZoomableImageView extends AppCompatImageView {
    private Matrix matrix = new Matrix();
    private float scaleFactor = 1.0f;
    private ScaleGestureDetector scaleGestureDetector;
    private PointF lastTouch = new PointF();
    private PointF startTouch = new PointF();
    private int mode = NONE;

    private static final int NONE = 0;
    private static final int DRAG = 1;
    private static final int ZOOM = 2;

    public ZoomableImageView(Context context, AttributeSet attrs) {
        super(context, attrs);
        init(context);
    }

    private void init(Context context) {
        setScaleType(ScaleType.MATRIX);
        matrix.setTranslate(1f, 1f);
        setImageMatrix(matrix);
        scaleGestureDetector = new ScaleGestureDetector(context, new ScaleListener());

        setOnTouchListener(new OnTouchListener() {
            @Override
            public boolean onTouch(View v, MotionEvent event) {
                scaleGestureDetector.onTouchEvent(event);

                PointF current = new PointF(event.getX(), event.getY());

                switch (event.getActionMasked()) {
                    case MotionEvent.ACTION_DOWN:
                        lastTouch.set(current);
                        startTouch.set(lastTouch);
                        mode = DRAG;
                        break;

                    case MotionEvent.ACTION_MOVE:
                        if (mode == DRAG) {
                            float dx = current.x - lastTouch.x;
                            float dy = current.y - lastTouch.y;
                            matrix.postTranslate(dx, dy);
                            setImageMatrix(matrix);
                            lastTouch.set(current.x, current.y);
                        }
                        break;

                    case MotionEvent.ACTION_POINTER_DOWN:
                        mode = ZOOM;
                        break;

                    case MotionEvent.ACTION_POINTER_UP:
                        mode = DRAG;
                        break;

                    case MotionEvent.ACTION_UP:
                        mode = NONE;
                        break;
                }

                return true;
            }
        });
    }

    private class ScaleListener extends ScaleGestureDetector.SimpleOnScaleGestureListener {
        @Override
        public boolean onScale(ScaleGestureDetector detector) {
            scaleFactor *= detector.getScaleFactor();
            scaleFactor = Math.max(1.0f, Math.min(scaleFactor, 5.0f)); // Set zoom limits
            matrix.setScale(scaleFactor, scaleFactor, getWidth() / 2f, getHeight() / 2f);
            setImageMatrix(matrix);
            return true;
        }
    }
}
