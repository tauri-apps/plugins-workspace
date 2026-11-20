// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.barcodescanner

import android.content.Context
import android.graphics.Canvas
import android.graphics.Matrix
import android.graphics.Paint
import android.util.AttributeSet
import android.view.View
import com.google.android.gms.common.internal.Preconditions

class GraphicOverlay: View {
    private val lock = Any()
    private val graphics: MutableList<Graphic> = ArrayList()

    private val transformationMatrix = Matrix()

    private var imageWidth = 0
    private var imageHeight = 0

    private var scaleFactor = 1.0f

    private var postScaleWidthOffset = 0f

    private var postScaleHeightOffset = 0f
    private var isImageFlipped = false
    private var needUpdateTransformation = true

    abstract class Graphic(private val overlay: GraphicOverlay) {
        abstract fun draw(canvas: Canvas?)
        protected fun drawRect(
            canvas: Canvas,
            left: Float,
            top: Float,
            right: Float,
            bottom: Float,
            paint: Paint?
        ) {
            canvas.drawRect(left, top, right, bottom, paint!!)
        }

        protected fun drawText(canvas: Canvas, text: String?, x: Float, y: Float, paint: Paint?) {
            canvas.drawText(text!!, x, y, paint!!)
        }

        /** Adjusts the supplied value from the image scale to the view scale.  */
        fun scale(imagePixel: Float): Float {
            return imagePixel * overlay.scaleFactor
        }

        val applicationContext
            get() = overlay.context.applicationContext

        fun isImageFlipped(): Boolean {
            return overlay.isImageFlipped
        }

        fun translateX(x: Float): Float {
            return if (overlay.isImageFlipped) {
                overlay.width - (scale(x) - overlay.postScaleWidthOffset)
            } else {
                scale(x) - overlay.postScaleWidthOffset
            }
        }

        fun translateY(y: Float): Float {
            return scale(y) - overlay.postScaleHeightOffset
        }

        fun getTransformationMatrix(): Matrix {
            return overlay.transformationMatrix
        }

        fun postInvalidate() {
            overlay.postInvalidate()
        }

        fun updatePaintColorByZValue(
            paint: Paint,
            canvas: Canvas,
            visualizeZ: Boolean,
            rescaleZForVisualization: Boolean,
            zInImagePixel: Float,
            zMin: Float,
            zMax: Float
        ) {
            if (!visualizeZ) {
                return
            }

            val zInScreenPixel = scale(zInImagePixel)
            if (zInScreenPixel < 0) {
                paint.setARGB(0, 0, 255, 0)
            } else {
                paint.setARGB(0, 0, 255, 0)
            }
        }
    }

    constructor(context: Context): super(context)

    constructor(context: Context, attrs: AttributeSet): super(context, attrs) {
        addOnLayoutChangeListener { _, _, _, _, _, _, _, _, _ ->
            needUpdateTransformation = true
        }
    }

    fun clear() {
        synchronized(lock) { graphics.clear() }
        postInvalidate()
    }

    fun add(graphic: Graphic) {
        synchronized(lock) { graphics.add(graphic) }
    }

    fun remove(graphic: Graphic) {
        synchronized(lock) { graphics.remove(graphic) }
        postInvalidate()
    }

    fun setImageSourceInfo(imageWidth: Int, imageHeight: Int, isFlipped: Boolean) {
        Preconditions.checkState(imageWidth > 0, "image width must be positive")
        Preconditions.checkState(imageHeight > 0, "image height must be positive")
        synchronized(lock) {
            this.imageWidth = imageWidth
            this.imageHeight = imageHeight
            isImageFlipped = isFlipped
            needUpdateTransformation = true
        }
        postInvalidate()
    }

    fun getImageWidth(): Int {
        return imageWidth
    }

    fun getImageHeight(): Int {
        return imageHeight
    }

    private fun updateTransformationIfNeeded() {
        if (!needUpdateTransformation || imageWidth <= 0 || imageHeight <= 0) {
            return
        }
        val viewAspectRatio = width.toFloat() / height
        val imageAspectRatio = imageWidth.toFloat() / imageHeight
        postScaleWidthOffset = 0f
        postScaleHeightOffset = 0f
        if (viewAspectRatio > imageAspectRatio) {
            // The image needs to be vertically cropped to be displayed in this view.
            scaleFactor = width.toFloat() / imageWidth
            postScaleHeightOffset = (width.toFloat() / imageAspectRatio - height) / 2
        } else {
            // The image needs to be horizontally cropped to be displayed in this view.
            scaleFactor = height.toFloat() / imageHeight
            postScaleWidthOffset = (height.toFloat() * imageAspectRatio - width) / 2
        }
        transformationMatrix.reset()
        transformationMatrix.setScale(scaleFactor, scaleFactor)
        transformationMatrix.postTranslate(-postScaleWidthOffset, -postScaleHeightOffset)
        if (isImageFlipped) {
            transformationMatrix.postScale(-1f, 1f, width / 2f, height / 2f)
        }
        needUpdateTransformation = false
    }

    override fun onDraw(canvas: Canvas) {
        super.onDraw(canvas)
        synchronized(lock) {
            updateTransformationIfNeeded()
            for (graphic in graphics) {
                graphic.draw(canvas)
            }
        }
    }
}