package eu.baysse.mykeyring.viewmodel

import eu.baysse.mykeyring.network.ApiDataSource
import eu.baysse.mykeyring.network.BaseDataSource
import javax.inject.Inject

class MainRepo @Inject constructor(private val apiDataSource: ApiDataSource): BaseDataSource() {

//    suspend fun sendNotification(name: String, token: String) = safeApiCall { apiDataSource.sendNotification(name, token) }

}
