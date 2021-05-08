package eu.baysse.mykeyring.view

import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import android.view.View
import android.view.WindowManager
import android.widget.*
//import androidx.activity.viewModels
//import androidx.lifecycle.Observer
//import androidx.lifecycle.ViewModel
import com.google.android.gms.tasks.OnCompleteListener
import com.google.firebase.messaging.FirebaseMessaging
import eu.baysse.mykeyring.R
//import eu.baysse.mykeyring.databinding.ActivityMainBinding
//import eu.baysse.mykeyring.helper.Resource
//import eu.baysse.mykeyring.viewmodel.MainViewModel
//import dagger.hilt.android.AndroidEntryPoint
import eu.baysse.mykeyring.helper.Init
import eu.baysse.mykeyring.helper.RustGreeting

//import kotlinx.android.synthetic.main.activity_main.*

//@AndroidEntryPoint
class MainActivity : AppCompatActivity() {

    //viewBinding
//    private var _binding: ActivityMainBinding? = null
//    private val binding get() = _binding!!
//    private var buttonSend: Button? = null
//    private var etName: EditText? = null
//    private var loadingProgress: ProgressBar? = null
//
//    private val mainViewModel: MainViewModel by viewModels()

    companion object {
        private const val TAG = "MyKeyring"
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        registerPush()

        setContentView(R.layout.activity_main)

        System.loadLibrary("mykeyring")
        // Initialise the librust, and so logs too
        Init().initlib()

        val r = RustGreeting().sayHello("WWWWWW")
        Log.d("rust", r)
        findViewById<EditText>(R.id.etName).setText(r)

//        _binding = ActivityMainBinding.inflate(layoutInflater)
//        val root = binding.root
//        setContentView(root)
//
//        initView()

    }

    private fun registerPush() {
        FirebaseMessaging.getInstance().token.addOnCompleteListener(OnCompleteListener { task ->
            if (!task.isSuccessful) {
                Log.w("token_failed", "Fetching FCM registration token failed", task.exception)
                return@OnCompleteListener
            }

            // Get new FCM registration token
            val token = task.result

            // Log and toast
            val msg = getString(R.string.msg_token_fmt, token)
            Log.d(TAG, msg)
            Toast.makeText(baseContext, msg, Toast.LENGTH_SHORT).show()
        })
    }

//    private fun initView(){
//
//        loadingProgress = binding.loading
//        buttonSend = binding.btnSend
//        etName = binding.etName
//
//        //listen to click event
//        buttonSend!!.setOnClickListener {
//
//            //hide button
//            buttonSend!!.visibility = View.GONE
//
//            //show progress bar
//            loadingProgress!!.visibility = View.VISIBLE
//
//            //register user
//            doSendNotification()
//        }
//
//    }
//
//    private fun doSendNotification(){
//
//        //get user notification token provided by firebase
//        FirebaseMessaging.getInstance().token.addOnCompleteListener(OnCompleteListener { task ->
//            if (!task.isSuccessful) {
//                Log.w("token_failed", "Fetching FCM registration token failed", task.exception)
//                return@OnCompleteListener
//            }
//
//            // Get new FCM registration token
//            val notificationToken = task.result
//            val messageString = etName!!.text.toString()
//
//            //store the user name
//            mainViewModel.doSendNotification(messageString, notificationToken!!)
//            setupObserver()
//        })
//
//    }
//
//    private fun setupObserver(){
//
//        //observe data obtained
//        mainViewModel.sendNotification.observe(this, Observer {
//
//            when(it.status){
//
//                Resource.Status.SUCCESS ->{
//
//                    if(it.data?.status == "success"){
//
//                        //stop progress bar
//                        loadingProgress!!.visibility = View.GONE
//                        buttonSend!!.visibility = View.VISIBLE
//
//                        //show toast message
//                        Toast.makeText(this, "Notification sent successfully", Toast.LENGTH_LONG).show()
//                    }
//
//                    else if(it.data?.status == "fail"){
//
//                        //stop progress bar
//                        loadingProgress!!.visibility = View.GONE
//                        buttonSend!!.visibility = View.VISIBLE
//
//                        //something went wrong, show error message
//                        Toast.makeText(this, it.message, Toast.LENGTH_LONG).show()
//
//                    }
//
//
//                }
//                Resource.Status.ERROR -> {
//
//                    Toast.makeText(this, it.message, Toast.LENGTH_LONG).show()
//
//                    loadingProgress!!.visibility = View.GONE
//                    buttonSend!!.visibility = View.VISIBLE
//
//                }
//                Resource.Status.LOADING -> {
//
//                    loadingProgress!!.visibility = View.VISIBLE
//                    buttonSend!!.visibility = View.GONE
//
//                }
//            }
//
//        })
//
//    }


}
