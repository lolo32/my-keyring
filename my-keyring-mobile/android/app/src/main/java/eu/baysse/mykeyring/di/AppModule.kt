package eu.baysse.mykeyring.di

//import com.google.android.datatransport.BuildConfig
//import com.google.gson.Gson
//import com.google.gson.GsonBuilder
//import eu.baysse.mykeyring.helper.EndPoints
//import eu.baysse.mykeyring.network.ApiDataSource
//import eu.baysse.mykeyring.network.ApiService
//import dagger.Module
//import dagger.Provides
//import dagger.hilt.InstallIn
//import dagger.hilt.android.components.ApplicationComponent
//import okhttp3.OkHttpClient
//import okhttp3.Protocol
//import okhttp3.logging.HttpLoggingInterceptor
//import retrofit2.Retrofit
//import retrofit2.converter.gson.GsonConverterFactory
//import retrofit2.converter.scalars.ScalarsConverterFactory
//import java.util.*
//import java.util.concurrent.TimeUnit
//import javax.inject.Singleton
//
//@Module
//@InstallIn(ApplicationComponent::class)
class AppModule {

//    //retrofit
//    @Provides
//    fun providesBaseUrl() = EndPoints.BASE_URL
//
//    @Provides
//    fun providesGson(): Gson = GsonBuilder().setLenient().create()
//
//    @Provides
//    @Singleton
//    fun provideRetrofit(gson: Gson) : Retrofit = Retrofit.Builder()
//        .baseUrl(EndPoints.BASE_URL)
//        .client(
//            OkHttpClient.Builder().also { client ->
//                if (BuildConfig.DEBUG){
//                    val logging = HttpLoggingInterceptor()
//                    logging.setLevel(HttpLoggingInterceptor.Level.BODY)
//                    client.addInterceptor(logging)
//                    client.connectTimeout(120, TimeUnit.SECONDS)
//                    client.readTimeout(120, TimeUnit.SECONDS)
//                    client.protocols(Collections.singletonList(Protocol.HTTP_1_1))
//                }
//            }.build()
//        )
//        .addConverterFactory(ScalarsConverterFactory.create())
//        .addConverterFactory(GsonConverterFactory.create(gson))
//        .build()
//
//    @Provides
//    @Singleton
//    fun provideApiService(retrofit: Retrofit): ApiService = retrofit.create(ApiService::class.java)
//
//    @Provides
//    @Singleton
//    fun provideApiDataSource(apiService: ApiService) = ApiDataSource(apiService)


}
