package eu.baysse.mykeyring.network

import javax.inject.Inject

class ApiDataSource @Inject constructor(private val apiService: ApiService) {

    //suspend fun sendNotification(name: String, token: String) = apiService.sendNotification(name, token)

}
