package de.selfmade4u.rustapplication

import android.os.Bundle
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    companion object {
        init {
            System.loadLibrary("backend")

            RustGreetings().greeting("Hi")
        }
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        val g = RustGreetings()
        val r = g.greeting("Rust")
        val tv: TextView = findViewById(R.id.test)
        tv.text = r
    }
}