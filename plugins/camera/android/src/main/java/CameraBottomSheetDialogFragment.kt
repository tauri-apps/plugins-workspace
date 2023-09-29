package app.tauri.camera

import android.annotation.SuppressLint
import android.app.Dialog
import android.content.DialogInterface
import android.graphics.Color
import android.view.View
import android.widget.LinearLayout
import android.widget.TextView
import androidx.coordinatorlayout.widget.CoordinatorLayout
import com.google.android.material.bottomsheet.BottomSheetBehavior
import com.google.android.material.bottomsheet.BottomSheetBehavior.BottomSheetCallback
import com.google.android.material.bottomsheet.BottomSheetDialogFragment

class CameraBottomSheetDialogFragment : BottomSheetDialogFragment() {
  fun interface BottomSheetOnSelectedListener {
    fun onSelected(index: Int)
  }

  fun interface BottomSheetOnCanceledListener {
    fun onCanceled()
  }

  private var selectedListener: BottomSheetOnSelectedListener? = null
  private var canceledListener: BottomSheetOnCanceledListener? = null
  private var options: List<String>? = null
  private var title: String? = null
  fun setTitle(title: String?) {
    this.title = title
  }

  fun setOptions(
    options: List<String>?,
    selectedListener: BottomSheetOnSelectedListener,
    canceledListener: BottomSheetOnCanceledListener
  ) {
    this.options = options
    this.selectedListener = selectedListener
    this.canceledListener = canceledListener
  }

  override fun onCancel(dialog: DialogInterface) {
    super.onCancel(dialog)
    if (canceledListener != null) {
      canceledListener!!.onCanceled()
    }
  }

  private val mBottomSheetBehaviorCallback: BottomSheetCallback = object : BottomSheetCallback() {
    override fun onStateChanged(bottomSheet: View, newState: Int) {
      if (newState == BottomSheetBehavior.STATE_HIDDEN) {
        dismiss()
      }
    }

    override fun onSlide(bottomSheet: View, slideOffset: Float) {}
  }

  @SuppressLint("RestrictedApi")
  override fun setupDialog(dialog: Dialog, style: Int) {
    super.setupDialog(dialog, style)
    if (options == null || options!!.size == 0) {
      return
    }
    val scale = resources.displayMetrics.density
    val layoutPaddingDp16 = 16.0f
    val layoutPaddingDp12 = 12.0f
    val layoutPaddingDp8 = 8.0f
    val layoutPaddingPx16 = (layoutPaddingDp16 * scale + 0.5f).toInt()
    val layoutPaddingPx12 = (layoutPaddingDp12 * scale + 0.5f).toInt()
    val layoutPaddingPx8 = (layoutPaddingDp8 * scale + 0.5f).toInt()
    val parentLayout = CoordinatorLayout(requireContext())
    val layout = LinearLayout(context)
    layout.orientation = LinearLayout.VERTICAL
    layout.setPadding(layoutPaddingPx16, layoutPaddingPx16, layoutPaddingPx16, layoutPaddingPx16)
    val ttv = TextView(context)
    ttv.setTextColor(Color.parseColor("#757575"))
    ttv.setPadding(layoutPaddingPx8, layoutPaddingPx8, layoutPaddingPx8, layoutPaddingPx8)
    ttv.text = title
    layout.addView(ttv)
    for (i in options!!.indices) {
      val tv = TextView(context)
      tv.setTextColor(Color.parseColor("#000000"))
      tv.setPadding(layoutPaddingPx12, layoutPaddingPx12, layoutPaddingPx12, layoutPaddingPx12)
      tv.text = options!![i]
      tv.setOnClickListener {
        if (selectedListener != null) {
          selectedListener!!.onSelected(i)
        }
        dismiss()
      }
      layout.addView(tv)
    }
    parentLayout.addView(layout.rootView)
    dialog.setContentView(parentLayout.rootView)
    val params = (parentLayout.parent as View).layoutParams as CoordinatorLayout.LayoutParams
    val behavior = params.behavior
    if (behavior != null && behavior is BottomSheetBehavior<*>) {
      behavior.addBottomSheetCallback(mBottomSheetBehaviorCallback)
    }
  }
}
