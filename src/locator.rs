
/*
fn locate_aruco_marker() {
  std::vector< int > ids;
  std::vector< std::vector< cv::Point2f > > corners, rejected;
  std::vector< cv::Vec3d > rvecs, tvecs;
  cv::Ptr<cv::aruco::Dictionary> dict = cv::aruco::getPredefinedDictionary(cv::aruco::DICT_4X4_50);
  cv::Ptr<cv::aruco::DetectorParameters> params = cv::aruco::DetectorParameters::create();
  
  // aruco lib can undistort for us so work on the original image...
  cv::aruco::detectMarkers(original, dict, corners, ids, params, rejected, cameraMatrix, distCoeffs);
  
  // draw onto image (for debugging purposes)
  cv::aruco::drawDetectedMarkers(original, corners);
  
  if (ids.size() == 1) {
      printf("found 1 marker, top-left corner %d, %d\n", (int) corners[0][0].x, (int) corners[0][0].y);
      
      cv::aruco::estimatePoseSingleMarkers(corners, markerSize, cameraMatrix, distCoeffs, rvecs,
                                            tvecs);
      
      // TODO - sometimes the X axis is flipped - need a way to detect this and retry/fail
      // or just flip the axis if that fixes everything
      
      for (int i=0 ; i<ids.size() ; i++) {
          cv::aruco::drawAxis(original, cameraMatrix, distCoeffs, rvecs[i], tvecs[i], 0.03);
      }
      
      cv::imwrite("/Users/tom/Pictures/detectedArucoMarkers.jpg", original);
      
      rvec.at<double>(0) = rvecs[0][0];
      rvec.at<double>(1) = rvecs[0][1];
      rvec.at<double>(2) = rvecs[0][2];
      
      tvec.at<double>(0) = tvecs[0][0];
      tvec.at<double>(1) = tvecs[0][1];
      tvec.at<double>(2) = tvecs[0][2];
  } else {
      Err("")
  }




}*/
