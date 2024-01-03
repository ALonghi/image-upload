"use client"
import ImageUpload from '@/components/ImageUpload'
import axios from 'axios'
import { error } from 'console'
import { useState } from 'react'
export interface Image {
  public_url: string,
  object_key: string
}

export default function Home() {
  const [response, setResponse] = useState<string>(``)
  const [images, setImages] = useState<Image[]>([])
  const testData = () => {
    axios
      .get(`${process.env.NEXT_PUBLIC_API_URL}/`)
      .then(res => setResponse(JSON.stringify(res.data, null, 2)))
      .catch(err => alert(err))
  }

  const listObjects = async () => {
    await axios
      .get(`${process.env.NEXT_PUBLIC_API_URL}/list`)
      .then(res => {
        setResponse(JSON.stringify(res.data, null, 2))
        setImages(res.data.data)
      })
      .catch(err => alert(err))
  }

  const addImage = (imageUrl) => {
    setImages(prev => [...prev, imageUrl])
  }

  const deleteImg = async (object_key) => {
    axios.post(`${process.env.NEXT_PUBLIC_API_URL}/delete`, {
      file_name: object_key
    })
      .then(res => {
        alert(res.data)
      })
      .catch(error => alert(error))
  }

  return (
    <main className="flex min-h-screen flex-col items-center justify-center gap-y-12 p-24">
      <div className="z-10 max-w-5xl w-full items-center justify-between font-mono text-sm lg:flex h-fit">
        <p>Backend url: {process.env.NEXT_PUBLIC_API_URL}</p>
        <ImageUpload setResponse={setResponse} addImage={addImage} />
        <button type="button"
          onClick={() => testData()}
          className='disabled:cursor-not-allowed disabled:border-red-600 py-2 px-8 rounded-lg border-gray-700 border hover:bg-gray-700 hover:text-white'>
          Test call</button>

        <button type="button"
          onClick={() => listObjects()}
          className='disabled:cursor-not-allowed disabled:border-red-600 py-2 px-8 rounded-lg border-gray-700 border hover:bg-gray-700 hover:text-white'>
          List objects</button>
      </div>
      <div className='mx-auto bg-gray-200 rounded-lg p-4'>
        <pre className='text-xs'>{response}</pre>
      </div>
      <p>Length: {images.length || 0}</p>
      {images.length > 0 && (
        <div className='mt-2 w-full flex flex-wrap gap-3 bg-gray-200 rounded-lg p-4'>
          {images.map((image, index) =>
            <div key={index} className='w-44 h-fit relative'>
              <p className='absolute cursor-pointer text-red-500 font-bold top-1 right-2 z-20 text-lg' onClick={() => deleteImg(image.object_key)}>x</p>
              <img src={image.public_url} className='w-44 h-auto object-cover z-10' />
            </div>
          )}
        </div>
      )}
    </main>
  )
}
