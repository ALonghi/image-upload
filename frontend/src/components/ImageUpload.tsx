"use client";
import React, { useState } from "react";
import axios from "axios";

const ImageUpload = ({ setResponse, addImage }) => {
  const [image, setImage] = useState(null);

  const handleImageChange = (e) => {
    setImage(e.target.files[0]);
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    const formData = new FormData();
    formData.append("image", image);

    try {
      console.log(`Calling backend with ${JSON.stringify(image, null, 2)}..`);
      await axios
        .post(`${process.env.NEXT_PUBLIC_API_URL}/upload`, formData, {
          headers: {
            "Content-Type": "multipart/form-data",
          },
        })
        .then((res) => {
          setResponse(JSON.stringify(res.data, null, 2));
          addImage(res.data.data);
        });
      alert("Image uploaded successfully");
    } catch (error) {
      console.error("Error uploading image", error);
      alert(error);
    }
  };

  return (
    <form onSubmit={handleSubmit} className="flex gap-x-8">
      <input type="file" onChange={handleImageChange} className="" />
      <button
        type="submit"
        disabled={image === null}
        className="disabled:cursor-not-allowed disabled:border-red-600 py-2 px-8 rounded-lg border-gray-700 border hover:bg-gray-700 hover:text-white"
      >
        Upload Image
      </button>
    </form>
  );
};

export default ImageUpload;
